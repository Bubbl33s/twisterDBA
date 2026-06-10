use std::collections::HashMap;

use crate::db::backend::{DbBackend, EngineType};
use crate::db::mysql::{
    execute_mysql_query, load_mysql_columns, load_mysql_schema, load_mysql_table_details,
    load_mysql_table_info,
};
use crate::db::pg_exec::execute_pg_query;
use crate::db::pg_schema::{
    load_pg_columns, load_pg_schema, load_pg_table_details, load_pg_table_info,
};
use crate::db::sqlite::{
    execute_sqlite_query, load_sqlite_columns, load_sqlite_schema, load_sqlite_table_details,
    load_sqlite_table_info,
};
use crate::events::{AppEvent, DbEvent};
use secrecy::SecretString;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub enum DbCommand {
    Connect {
        connection_name: String,
        dsn: SecretString,
        engine_type: EngineType,
    },
    Disconnect {
        connection_name: String,
    },
    LoadSchema {
        connection_name: String,
    },
    #[allow(dead_code)]
    LoadColumns {
        connection_name: String,
        schema: String,
        table: String,
    },
    LoadTableDetails {
        connection_name: String,
        schema: String,
        table: String,
    },
    LoadTableInfo {
        connection_name: String,
        schema: String,
        table: String,
    },
    #[allow(dead_code)]
    ExecuteQuery {
        connection_name: String,
        sql: String,
        cancel: CancellationToken,
        auto_paginate: bool,
        page_size: usize,
    },
    #[allow(dead_code)]
    FetchNextPage {
        connection_name: String,
        page: usize,
        sql: String,
        cancel: CancellationToken,
    },
}

pub struct DbClient {
    backends: HashMap<String, DbBackend>,
    command_rx: mpsc::UnboundedReceiver<DbCommand>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl DbClient {
    pub fn new(
        command_rx: mpsc::UnboundedReceiver<DbCommand>,
        event_tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Self {
        Self { backends: HashMap::new(), command_rx, event_tx }
    }

    pub async fn run(&mut self) {
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                DbCommand::Connect { connection_name, dsn, engine_type } => {
                    self.handle_connect(&connection_name, dsn, engine_type).await;
                },
                DbCommand::Disconnect { connection_name } => {
                    self.handle_disconnect(&connection_name).await;
                },
                DbCommand::LoadSchema { connection_name } => {
                    self.handle_load_schema(&connection_name).await;
                },
                DbCommand::LoadColumns { connection_name, schema, table } => {
                    self.handle_load_columns(&connection_name, &schema, &table).await;
                },
                DbCommand::LoadTableDetails { connection_name, schema, table } => {
                    self.handle_load_table_details(&connection_name, &schema, &table).await;
                },
                DbCommand::LoadTableInfo { connection_name, schema, table } => {
                    self.handle_load_table_info(&connection_name, &schema, &table).await;
                },
                DbCommand::ExecuteQuery { connection_name, sql, cancel, .. } => {
                    self.handle_execute_query(&connection_name, sql, cancel).await;
                },
                DbCommand::FetchNextPage { connection_name, sql, cancel, .. } => {
                    self.handle_execute_query(&connection_name, sql, cancel).await;
                },
            }
        }
    }

    async fn handle_connect(
        &mut self,
        connection_name: &str,
        dsn: SecretString,
        engine_type: EngineType,
    ) {
        if let Some(old) = self.backends.remove(connection_name) {
            old.close().await;
        }

        match DbBackend::connect(&dsn, engine_type).await {
            Ok(backend) => {
                self.backends.insert(connection_name.to_string(), backend);
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::Connected {
                    connection_name: connection_name.to_string(),
                }));
            },
            Err(e) => {
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                    connection_name: connection_name.to_string(),
                    message: e,
                }));
            },
        }
    }

    async fn handle_disconnect(&mut self, connection_name: &str) {
        if let Some(backend) = self.backends.remove(connection_name) {
            backend.close().await;
        }
        let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::Disconnected {
            connection_name: connection_name.to_string(),
        }));
    }

    async fn handle_load_schema(&mut self, connection_name: &str) {
        let backend = match self.backends.get(connection_name) {
            Some(b) => b,
            None => {
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                    connection_name: connection_name.to_string(),
                    message: format!("Unknown connection: {connection_name}"),
                }));
                return;
            },
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_schema(pool).await {
                Ok(tree) => {
                    info!("Schema loaded: {} schemas", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded {
                        connection_name: connection_name.to_string(),
                        nodes: tree,
                    }));
                },
                Err(e) => {
                    error!("LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                        connection_name: connection_name.to_string(),
                        message: e,
                    }));
                },
            },
            DbBackend::Mysql(pool) => match load_mysql_schema(pool).await {
                Ok(tree) => {
                    info!("MySQL schema loaded: {} schemas", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded {
                        connection_name: connection_name.to_string(),
                        nodes: tree,
                    }));
                },
                Err(e) => {
                    error!("MySQL LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                        connection_name: connection_name.to_string(),
                        message: e,
                    }));
                },
            },
            DbBackend::Sqlite(pool) => match load_sqlite_schema(pool).await {
                Ok(tree) => {
                    info!("SQLite schema loaded: {} tables", tree.len());
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::SchemaLoaded {
                        connection_name: connection_name.to_string(),
                        nodes: tree,
                    }));
                },
                Err(e) => {
                    error!("SQLite LoadSchema query failed: {e}");
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                        connection_name: connection_name.to_string(),
                        message: e,
                    }));
                },
            },
            DbBackend::Disconnected => {
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ConnectionFailed {
                    connection_name: connection_name.to_string(),
                    message: "Not connected".into(),
                }));
            },
        }
    }

    async fn handle_load_columns(&mut self, connection_name: &str, schema: &str, table: &str) {
        let backend = match self.backends.get(connection_name) {
            Some(b) => b,
            None => return,
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_columns(pool, schema, table).await {
                Ok(columns) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::ColumnsLoaded {
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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

    async fn handle_load_table_details(
        &mut self,
        connection_name: &str,
        schema: &str,
        table: &str,
    ) {
        let backend = match self.backends.get(connection_name) {
            Some(b) => b,
            None => return,
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_table_details(pool, schema, table).await {
                Ok(details) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableDetailsLoaded {
                        connection_name: connection_name.to_string(),
                        schema: schema.to_string(),
                        table: table.to_string(),
                        details,
                    }));
                },
                Err(e) => {
                    error!("LoadTableDetails failed: {e}");
                },
            },
            DbBackend::Mysql(pool) => match load_mysql_table_details(pool, schema, table).await {
                Ok(details) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableDetailsLoaded {
                        connection_name: connection_name.to_string(),
                        schema: schema.to_string(),
                        table: table.to_string(),
                        details,
                    }));
                },
                Err(e) => {
                    error!("MySQL LoadTableDetails failed: {e}");
                },
            },
            DbBackend::Sqlite(pool) => match load_sqlite_table_details(pool, schema, table).await {
                Ok(details) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableDetailsLoaded {
                        connection_name: connection_name.to_string(),
                        schema: schema.to_string(),
                        table: table.to_string(),
                        details,
                    }));
                },
                Err(e) => {
                    error!("SQLite LoadTableDetails failed: {e}");
                },
            },
            DbBackend::Disconnected => {},
        }
    }

    async fn handle_load_table_info(&mut self, connection_name: &str, schema: &str, table: &str) {
        let backend = match self.backends.get(connection_name) {
            Some(b) => b,
            None => return,
        };

        match backend {
            DbBackend::Pg(pool) => match load_pg_table_info(pool, schema, table).await {
                Ok(info) => {
                    let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::TableInfoLoaded {
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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
                        connection_name: connection_name.to_string(),
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

    async fn handle_execute_query(
        &mut self,
        connection_name: &str,
        sql: String,
        cancel: CancellationToken,
    ) {
        let backend = match self.backends.get(connection_name) {
            Some(b) => b,
            None => {
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::QueryError {
                    connection_name: connection_name.to_string(),
                    message: format!("Unknown connection: {connection_name}"),
                }));
                return;
            },
        };

        let cn = connection_name.to_string();
        match backend {
            DbBackend::Pg(pool) => {
                execute_pg_query(pool, sql, cancel, &self.event_tx, &cn).await;
            },
            DbBackend::Mysql(pool) => {
                execute_mysql_query(pool, sql, cancel, &self.event_tx, &cn).await;
            },
            DbBackend::Sqlite(pool) => {
                execute_sqlite_query(pool, sql, cancel, &self.event_tx, &cn).await;
            },
            DbBackend::Disconnected => {
                let _ = self.event_tx.send(AppEvent::DbEvent(DbEvent::QueryError {
                    connection_name: connection_name.to_string(),
                    message: "Not connected".into(),
                }));
            },
        }
    }
}
