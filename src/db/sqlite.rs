use crate::events::{AppEvent, ColumnInfo, DbEvent, TableInfo};
use crate::explorer::SchemaNode;
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

    let mut children: Vec<SchemaNode> = Vec::new();

    for row in &rows {
        let name: String = row.get("name");
        let obj_type: String = row.get("type");

        let node = match obj_type.as_str() {
            "view" => SchemaNode::View { schema: "main".to_string(), name },
            _ => SchemaNode::Table {
                schema: "main".to_string(),
                name,
                expanded: false,
                loaded: false,
                children: Vec::new(),
            },
        };

        children.push(node);
    }

    Ok(children)
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
