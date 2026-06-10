use crate::events::{
    AppEvent, ColumnInfo, DbEvent, ForeignKeyInfo, IndexInfo, KeyInfo, TableDetails, TableInfo,
};
use crate::explorer::{FolderKind, SchemaNode};
use crate::result::ColumnMeta;
use futures_util::StreamExt;
use sqlx::{Column, Row, SqlitePool, TypeInfo};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

pub async fn load_sqlite_schema(pool: &SqlitePool) -> Result<Vec<SchemaNode>, String> {
    let rows = sqlx::query(
        "SELECT name, type FROM sqlite_master
         WHERE type IN ('table', 'view')
         AND name NOT LIKE 'sqlite_%'
         ORDER BY type, name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("SQLite schema query: {e}"))?;

    let mut tables: Vec<SchemaNode> = Vec::new();
    let mut views: Vec<SchemaNode> = Vec::new();

    for row in &rows {
        let name: String = row.get("name");
        let obj_type: String = row.get("type");

        match obj_type.as_str() {
            "view" => views.push(SchemaNode::View { schema: "main".to_string(), name }),
            _ => tables.push(SchemaNode::Table {
                schema: "main".to_string(),
                name,
                expanded: false,
                loaded: false,
                children: Vec::new(),
            }),
        }
    }

    let mut children = Vec::new();
    if !tables.is_empty() {
        children.push(SchemaNode::ObjectFolder {
            kind: FolderKind::Tables,
            expanded: false,
            loaded: true,
            children: tables,
        });
    }
    if !views.is_empty() {
        children.push(SchemaNode::ObjectFolder {
            kind: FolderKind::Views,
            expanded: false,
            loaded: true,
            children: views,
        });
    }

    Ok(vec![SchemaNode::Schema { name: "main".to_string(), expanded: false, children }])
}

pub async fn load_sqlite_columns(
    pool: &SqlitePool,
    _schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, String> {
    let query_string = format!("PRAGMA table_info('{table}')");
    let rows =
        sqlx::query(sqlx::AssertSqlSafe(query_string)).fetch_all(pool).await.map_err(|e| {
            error!("SQLite LoadColumns query failed: {e}");
            format!("{e}")
        })?;

    let columns: Vec<ColumnInfo> = rows
        .iter()
        .map(|row| {
            let name: String = row.get("name");
            let data_type: String = row.get("type");
            let notnull: i32 = row.get("notnull");
            let pk: i32 = row.get("pk");
            ColumnInfo {
                name,
                data_type: if data_type.is_empty() {
                    "TEXT".to_string()
                } else {
                    data_type.to_uppercase()
                },
                nullable: notnull == 0,
                is_primary_key: pk != 0,
            }
        })
        .collect();

    Ok(columns)
}

pub async fn load_sqlite_table_details(
    pool: &SqlitePool,
    _schema: &str,
    table: &str,
) -> Result<TableDetails, String> {
    let columns = load_sqlite_columns(pool, _schema, table).await?;

    let pk_columns: Vec<String> =
        columns.iter().filter(|c| c.is_primary_key).map(|c| c.name.clone()).collect();
    let keys = if pk_columns.is_empty() {
        Vec::new()
    } else {
        vec![KeyInfo { name: "primary".to_string(), columns: pk_columns }]
    };

    let fk_query = format!("PRAGMA foreign_key_list('{table}')");
    let fk_rows = sqlx::query(sqlx::AssertSqlSafe(fk_query))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("SQLite FK query: {e}"))?;

    let mut fk_map: std::collections::BTreeMap<i32, (String, String, Vec<String>, Vec<String>)> =
        std::collections::BTreeMap::new();
    for row in &fk_rows {
        let id: i32 = row.get("id");
        let ref_table: String = row.get("table");
        let from_col: String = row.get("from");
        let to_col: String = row.get("to");
        let entry = fk_map
            .entry(id)
            .or_insert_with(|| (format!("fk_{id}"), ref_table, Vec::new(), Vec::new()));
        entry.2.push(from_col);
        entry.3.push(to_col);
    }
    let foreign_keys: Vec<ForeignKeyInfo> = fk_map
        .into_values()
        .map(|(name, ref_table, columns, ref_columns)| ForeignKeyInfo {
            name,
            columns,
            ref_table,
            ref_columns,
        })
        .collect();

    let idx_list_query = format!("PRAGMA index_list('{table}')");
    let idx_rows = sqlx::query(sqlx::AssertSqlSafe(idx_list_query))
        .fetch_all(pool)
        .await
        .map_err(|e| format!("SQLite index list query: {e}"))?;

    let mut indexes: Vec<IndexInfo> = Vec::new();
    for row in &idx_rows {
        let idx_name: String = row.get("name");
        let unique: bool = row.get("unique");

        let idx_info_query = format!("PRAGMA index_info('{idx_name}')");
        let idx_info_rows = sqlx::query(sqlx::AssertSqlSafe(idx_info_query))
            .fetch_all(pool)
            .await
            .map_err(|e| format!("SQLite index info query: {e}"))?;

        let idx_columns: Vec<String> =
            idx_info_rows.iter().map(|r| r.get::<String, _>("name")).collect();

        indexes.push(IndexInfo {
            name: idx_name,
            columns: idx_columns,
            is_unique: unique,
            is_primary: false,
        });
    }

    Ok(TableDetails { columns, keys, foreign_keys, indexes })
}

pub async fn load_sqlite_table_info(
    pool: &SqlitePool,
    _schema: &str,
    table: &str,
) -> Result<TableInfo, String> {
    let ddl = sqlx::query_scalar::<_, String>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let ddl = if ddl.is_none() {
        sqlx::query_scalar::<_, String>(
            "SELECT sql FROM sqlite_master WHERE type = 'view' AND name = ?",
        )
        .bind(table)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    } else {
        ddl
    };

    let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", table.replace('\"', "\"\""));
    let row_count = match sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(count_sql))
        .fetch_optional(pool)
        .await
    {
        Ok(Some(s)) => s.parse::<u64>().ok(),
        _ => None,
    };

    let table_size = row_count.map(|n| {
        let estimated = n.saturating_mul(256);
        format_sqlite_size(estimated)
    });

    Ok(TableInfo { ddl, row_count, table_size })
}

fn format_sqlite_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx + 1 < UNITS.len() {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("~{:.1} {}", size, UNITS[unit_idx])
}

pub async fn execute_sqlite_query(
    pool: &SqlitePool,
    sql: String,
    cancel: CancellationToken,
    tx: &mpsc::UnboundedSender<AppEvent>,
    connection_name: &str,
) {
    let _ = tx.send(AppEvent::DbEvent(DbEvent::QueryStarted {
        connection_name: connection_name.to_string(),
    }));

    let start = std::time::Instant::now();
    let mut rows_affected: u64 = 0;
    let mut columns_sent = false;
    let mut stream = sqlx::raw_sql(sqlx::AssertSqlSafe(sql)).fetch(pool);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = tx.send(AppEvent::DbEvent(DbEvent::QueryCancelled {
                    connection_name: connection_name.to_string(),
                }));
                return;
            }
            result = stream.next() => {
                match result {
                    Some(Ok(row)) => {
                        if !columns_sent {
                            let col_meta: Vec<ColumnMeta> = row.columns()
                                .iter()
                                .map(|c| ColumnMeta {
                                    name: c.name().to_string(),
                                    data_type: c.type_info().name().to_string(),
                                    is_primary_key: false,
                                })
                                .collect();
                            let _ = tx.send(AppEvent::DbEvent(
                                DbEvent::ResultColumns {
                                    connection_name: connection_name.to_string(),
                                    columns: col_meta,
                                },
                            ));
                            columns_sent = true;
                        }

                        let columns = row.columns();
                        let values: Vec<String> = columns
                            .iter()
                            .enumerate()
                            .map(|(i, _col)| {
                                row.try_get::<String, _>(i)
                                    .unwrap_or_else(|_| "?".into())
                            })
                            .collect();
                        let _ = tx.send(AppEvent::DbEvent(DbEvent::QueryRow {
                            connection_name: connection_name.to_string(),
                            cells: values,
                        }));
                        rows_affected += 1;
                    }
                    Some(Err(e)) => {
                        let _ = tx.send(AppEvent::DbEvent(
                            DbEvent::QueryError {
                                connection_name: connection_name.to_string(),
                                message: format!("SQLite: {e}"),
                            },
                        ));
                        return;
                    }
                    None => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let _ = tx.send(AppEvent::DbEvent(DbEvent::QueryCompleted {
                            connection_name: connection_name.to_string(),
                            _rows_affected: rows_affected,
                            _duration_ms: duration_ms,
                        }));
                        return;
                    }
                }
            }
        }
    }
}
