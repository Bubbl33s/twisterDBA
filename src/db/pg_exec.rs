use crate::events::DbEvent;
use crate::result::ColumnMeta;
use futures_util::StreamExt;
use sqlx::{Column, PgPool, Row, TypeInfo};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub async fn execute_pg_query(
    pool: &PgPool,
    sql: String,
    cancel: CancellationToken,
    tx: &mpsc::UnboundedSender<crate::events::AppEvent>,
) {
    let _ = tx.send(crate::events::AppEvent::DbEvent(DbEvent::QueryStarted));

    let start = std::time::Instant::now();
    let mut rows_affected: u64 = 0;
    let mut columns_sent = false;
    let mut stream = sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str())).fetch(pool);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = tx.send(crate::events::AppEvent::DbEvent(DbEvent::QueryCancelled));
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
                            let _ = tx.send(crate::events::AppEvent::DbEvent(
                                DbEvent::ResultColumns(col_meta),
                            ));
                            columns_sent = true;
                        }

                        let columns = row.columns();
                        let values: Vec<String> = columns
                            .iter()
                            .enumerate()
                            .map(|(i, _col)| {
                                row.try_get::<String, _>(i)
                                    .or_else(|_| row.try_get::<i64, _>(i).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<i32, _>(i).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<f64, _>(i).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<bool, _>(i).map(|v| v.to_string()))
                                    .unwrap_or_else(|_| "?".into())
                            })
                            .collect();
                        let _ = tx.send(crate::events::AppEvent::DbEvent(DbEvent::QueryRow(values)));
                        rows_affected += 1;
                    }
                    Some(Err(e)) => {
                        let _ = tx.send(crate::events::AppEvent::DbEvent(
                            DbEvent::QueryError(format!("{e}")),
                        ));
                        return;
                    }
                    None => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let _ = tx.send(crate::events::AppEvent::DbEvent(DbEvent::QueryCompleted {
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
