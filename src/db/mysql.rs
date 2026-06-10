use crate::events::{AppEvent, ColumnInfo, DbEvent, TableInfo};
use crate::explorer::SchemaNode;
use crate::result::ColumnMeta;
use futures_util::StreamExt;
use sqlx::{Column, MySqlPool, Row, TypeInfo};
use std::collections::BTreeMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

pub async fn load_mysql_schema(pool: &MySqlPool) -> Result<Vec<SchemaNode>, String> {
    let rows = sqlx::query(
        "SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE
         FROM information_schema.TABLES
         WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys')
         ORDER BY TABLE_SCHEMA, TABLE_NAME",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("MySQL schema query: {e}"))?;

    let mut schemas: BTreeMap<String, Vec<SchemaNode>> = BTreeMap::new();

    for row in &rows {
        let schema_name: String = row.get("TABLE_SCHEMA");
        let table_name: String = row.get("TABLE_NAME");
        let table_type: String = row.get("TABLE_TYPE");

        let node = match table_type.to_uppercase().as_str() {
            "VIEW" => SchemaNode::View { schema: schema_name.clone(), name: table_name },
            _ => SchemaNode::Table {
                schema: schema_name.clone(),
                name: table_name,
                expanded: false,
                loaded: false,
                children: Vec::new(),
            },
        };

        schemas.entry(schema_name).or_default().push(node);
    }

    let tree: Vec<SchemaNode> = schemas
        .into_iter()
        .map(|(name, children)| SchemaNode::Schema { name, expanded: false, children })
        .collect();

    Ok(tree)
}

pub async fn load_mysql_columns(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, String> {
    let rows = sqlx::query(
        "SELECT c.COLUMN_NAME, c.COLUMN_TYPE, c.IS_NULLABLE,
                CASE WHEN ku.COLUMN_NAME IS NOT NULL THEN true ELSE false END as is_primary_key
         FROM information_schema.COLUMNS c
         LEFT JOIN information_schema.KEY_COLUMN_USAGE ku
             ON c.TABLE_SCHEMA = ku.TABLE_SCHEMA
             AND c.TABLE_NAME = ku.TABLE_NAME
             AND c.COLUMN_NAME = ku.COLUMN_NAME
             AND ku.CONSTRAINT_NAME = 'PRIMARY'
         WHERE c.TABLE_SCHEMA = ? AND c.TABLE_NAME = ?
         ORDER BY c.ORDINAL_POSITION",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("MySQL LoadColumns query failed: {e}");
        format!("{e}")
    })?;

    let columns: Vec<ColumnInfo> = rows
        .iter()
        .map(|row| {
            let name: String = row.get("COLUMN_NAME");
            let data_type: String = row.get("COLUMN_TYPE");
            let nullable_str: String = row.get("IS_NULLABLE");
            let is_pk: bool = row.get("is_primary_key");
            ColumnInfo { name, data_type, nullable: nullable_str == "YES", is_primary_key: is_pk }
        })
        .collect();

    Ok(columns)
}

pub async fn load_mysql_table_info(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
) -> Result<TableInfo, String> {
    let ddl = reconstruct_mysql_ddl(pool, schema, table).await;

    let row_count = match sqlx::query_scalar::<_, i64>(
        "SELECT TABLE_ROWS FROM information_schema.TABLES
         WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?",
    )
    .bind(schema)
    .bind(table)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(n)) if n >= 0 => Some(n as u64),
        Ok(_) => None,
        Err(_) => None,
    };

    let table_size = match sqlx::query_scalar::<_, i64>(
        "SELECT data_length + index_length FROM information_schema.TABLES
         WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?",
    )
    .bind(schema)
    .bind(table)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(bytes)) if bytes >= 0 => Some(format_mysql_size(bytes as u64)),
        Ok(_) => None,
        Err(_) => None,
    };

    Ok(TableInfo { ddl, row_count, table_size })
}

async fn reconstruct_mysql_ddl(pool: &MySqlPool, schema: &str, table: &str) -> Option<String> {
    let columns = sqlx::query(
        "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, EXTRA,
                COLUMN_KEY
         FROM information_schema.COLUMNS
         WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
         ORDER BY ORDINAL_POSITION",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await
    .ok()?;

    if columns.is_empty() {
        return None;
    }

    let mut ddl = format!("CREATE TABLE `{}`.`{}` (\n", schema, table);
    let mut col_defs: Vec<String> = Vec::new();
    for row in &columns {
        let col_name: String = row.get("COLUMN_NAME");
        let col_type: String = row.get("COLUMN_TYPE");
        let nullable: String = row.get("IS_NULLABLE");
        let default: Option<String> = row.try_get("COLUMN_DEFAULT").ok();
        let extra: String = row.try_get("EXTRA").unwrap_or_default();
        let col_key: String = row.try_get("COLUMN_KEY").unwrap_or_default();

        let mut def = format!("  `{}` {}", col_name, col_type);
        if nullable == "NO" {
            def.push_str(" NOT NULL");
        }
        if let Some(ref d) = default {
            if d.to_uppercase() == "CURRENT_TIMESTAMP" || extra.contains("on update") {
                def.push_str(&format!(" DEFAULT {}", d));
            } else {
                def.push_str(&format!(" DEFAULT '{}'", d));
            }
        }
        if !extra.is_empty() && extra != "on update CURRENT_TIMESTAMP" {
            def.push_str(&format!(" {}", extra));
        }
        if col_key == "PRI" {
            def.push_str(" PRIMARY KEY");
        }
        col_defs.push(def);
    }
    ddl.push_str(&col_defs.join(",\n"));
    ddl.push_str("\n)");
    Some(ddl)
}

fn format_mysql_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx + 1 < UNITS.len() {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

pub async fn execute_mysql_query(
    pool: &MySqlPool,
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
    let mut stream = sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str())).fetch(pool);

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
                                message: format!("MySQL: {e}"),
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
