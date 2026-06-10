use crate::db::backend::{DbBackend, EngineType};
use crate::db::mysql::{
    execute_mysql_query, load_mysql_columns, load_mysql_schema, load_mysql_table_info,
};
use crate::db::pg_exec::execute_pg_query;
use crate::db::pg_schema::{load_pg_columns, load_pg_schema, load_pg_table_info};
use crate::db::sqlite::{
    execute_sqlite_query, load_sqlite_columns, load_sqlite_schema, load_sqlite_table_info,
};
use crate::events::{AppEvent, DbEvent};
use secrecy::SecretString;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub enum DbCommand {
    Connect {
        dsn: SecretString,
        engine_type: EngineType,
    },
    Disconnect,
    LoadSchema,
    LoadColumns {
        schema: String,
        table: String,
    },
    LoadTableInfo {
        schema: String,
        table: String,
    },
    #[allow(dead_code)]
    ExecuteQuery {
        sql: String,
        cancel: CancellationToken,
        auto_paginate: bool,
        page_size: usize,
    },
    #[allow(dead_code)]
    FetchNextPage {
        page: usize,
        sql: String,
        cancel: CancellationToken,
    },
}

pub struct DbClient {
    backend: Option<DbBackend>,
    command_rx: mpsc::UnboundedReceiver<DbCommand>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl DbClient {
    pub fn new(
        command_rx: mpsc::UnboundedReceiver<DbCommand>,
        event_tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Self {
        Self { backend: None, command_rx, event_tx }
    }

    pub async fn run(&mut self) {
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                DbCommand::Connect { dsn, engine_type } => {
                    self.handle_connect(dsn, engine_type).await;
                },
                DbCommand::Disconnect => self.handle_disconnect().await,
                DbCommand::LoadSchema => self.handle_load_schema().await,
                DbCommand::LoadColumns { schema, table } => {
                    self.handle_load_columns(&schema, &table).await;
                },
                DbCommand::LoadTableInfo { schema, table } => {
                    self.handle_load_table_info(&schema, &table).await;
                },
                DbCommand::ExecuteQuery { sql, cancel, .. } => {
                    self.handle_execute_query(sql, cancel).await;
                },
                DbCommand::FetchNextPage { sql, cancel, .. } => {
                    self.handle_execute_query(sql, cancel).await;
                },
            }
        }
    }

    async fn handle_connect(&mut self, dsn: SecretString, engine_type: EngineType) {
        if let Some(backend) = self.backend.take() {
            backend.close().await;
        }

        match DbBackend::connect(&dsn, engine_type).await {
            Ok(backend) => {
                self.backend = Some(backend);
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::Connected));
            },
            Err(e) => {
                self.backend = None;
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed(e)));
            },
        }
    }

    async fn handle_disconnect(&mut self) {
        if let Some(backend) = self.backend.take() {
            backend.close().await;
        }
        let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::Disconnected));
    }

    async fn handle_load_schema(&mut self) {
        let backend = match &self.backend {
            Some(b) => b,
            None => {
                let _ = self
                    .event_tx
                    .send(AppEvent::DbEvent(DbEvent::ConnectionFailed("Not connected".into())));
                return;
            },
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_schema(pool).await {
                Ok(tree) => {
                    info!("Schema loaded: {} schemas", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded(tree)));
                },
                Err(e) => {
                    error!("LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed(e)));
                },
            },
            DbBackend::Mysql(pool) => match load_mysql_schema(pool).await {
                Ok(tree) => {
                    info!("MySQL schema loaded: {} schemas", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded(tree)));
                },
                Err(e) => {
                    error!("MySQL LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed(e)));
                },
            },
            DbBackend::Sqlite(pool) => match load_sqlite_schema(pool).await {
                Ok(tree) => {
                    info!("SQLite schema loaded: {} tables", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded(tree)));
                },
                Err(e) => {
                    error!("SQLite LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed(e)));
                },
            },
            DbBackend::Disconnected => {
                let _ = self
                    .event_tx
                    .send(AppEvent::DbEvent(DbEvent::ConnectionFailed("Not connected".into())));
            },
        }
    }

    async fn handle_load_columns(&mut self, schema: &str, table: &str) {
        let backend = match &self.backend {
            Some(b) => b,
            None => return,
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_columns(pool, schema, table).await {
                Ok(columns) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ColumnsLoaded {
                        schema: schema.to_string(),
                        table: table.to_string(),
                        columns,
                    }));
                },
                Err(e) => {
                    error!("LoadColumns failed: {e}");
                },
            },
            DbBackend::Mysql(pool) => match load_mysql_columns(pool, schema, table).await {
                Ok(columns) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ColumnsLoaded {
                        schema: schema.to_string(),
                        table: table.to_string(),
                        columns,
                    }));
                },
                Err(e) => {
                    error!("MySQL LoadColumns failed: {e}");
                },
            },
            DbBackend::Sqlite(pool) => match load_sqlite_columns(pool, schema, table).await {
                Ok(columns) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ColumnsLoaded {
                        schema: schema.to_string(),
                        table: table.to_string(),
                        columns,
                    }));
                },
                Err(e) => {
                    error!("SQLite LoadColumns failed: {e}");
                },
            },
            DbBackend::Disconnected => {},
        }
    }

    async fn handle_load_table_info(&mut self, schema: &str, table: &str) {
        let backend = match &self.backend {
            Some(b) => b,
            None => return,
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_table_info(pool, schema, table).await {
                Ok(info) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: info.ddl,
                        row_count: info.row_count,
                        table_size: info.table_size,
                    }));
                },
                Err(e) => {
                    error!("LoadTableInfo failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: Some(format!("Error: {e}")),
                        row_count: None,
                        table_size: None,
                    }));
                },
            },
            DbBackend::Mysql(pool) => match load_mysql_table_info(pool, schema, table).await {
                Ok(info) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: info.ddl,
                        row_count: info.row_count,
                        table_size: info.table_size,
                    }));
                },
                Err(e) => {
                    error!("MySQL LoadTableInfo failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: Some(format!("Error: {e}")),
                        row_count: None,
                        table_size: None,
                    }));
                },
            },
            DbBackend::Sqlite(pool) => match load_sqlite_table_info(pool, schema, table).await {
                Ok(info) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: info.ddl,
                        row_count: info.row_count,
                        table_size: info.table_size,
                    }));
                },
                Err(e) => {
                    error!("SQLite LoadTableInfo failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        _schema: schema.to_string(),
                        _table: table.to_string(),
                        ddl: Some(format!("Error: {e}")),
                        row_count: None,
                        table_size: None,
                    }));
                },
            },
            DbBackend::Disconnected => {},
        }
    }

    async fn handle_execute_query(&mut self, sql: String, cancel: CancellationToken) {
        let backend = match &self.backend {
            Some(b) => b,
            None => {
                let _ = self
                    .event_tx
                    .send(AppEvent::DbEvent(DbEvent::QueryError("Not connected".into())));
                return;
            },
        };

        match backend {
            DbBackend::Pg(pool) => {
                execute_pg_query(pool, sql, cancel, &self.event_tx).await;
            },
            DbBackend::Mysql(pool) => {
                execute_mysql_query(pool, sql, cancel, &self.event_tx).await;
            },
            DbBackend::Sqlite(pool) => {
                execute_sqlite_query(pool, sql, cancel, &self.event_tx).await;
            },
            DbBackend::Disconnected => {
                let _ = self
                    .event_tx
                    .send(AppEvent::DbEvent(DbEvent::QueryError("Not connected".into())));
            },
        }
    }
}
