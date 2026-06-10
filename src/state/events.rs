use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::backend::EngineType;
use crate::db::client::DbCommand;
use crate::events::DbEvent;
use crate::explorer::{DbSource, SchemaNode};
use crate::state::{AppState, ConnectionEntry, ConnectionStatus, PopupState};

fn timestamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl AppState {
    pub fn apply_db_event(&mut self, event: &DbEvent) {
        match event {
            DbEvent::Connected { connection_name } => {
                let (dsn, masked, engine_type) =
                    if let Some(entry) = self.connection_by_name(connection_name) {
                        match &entry.status {
                            ConnectionStatus::Connecting { dsn, masked } => {
                                (dsn.clone(), masked.clone(), entry.engine_type)
                            },
                            ConnectionStatus::Connected { dsn, masked } => {
                                (dsn.clone(), masked.clone(), entry.engine_type)
                            },
                            _ => (String::new(), entry.masked_dsn.clone(), entry.engine_type),
                        }
                    } else {
                        (String::new(), String::new(), EngineType::Postgres)
                    };

                if let Some(entry) = self.connection_by_name_mut(connection_name) {
                    entry.status =
                        ConnectionStatus::Connected { dsn: dsn.clone(), masked: masked.clone() };
                } else {
                    self.connections.push(ConnectionEntry {
                        name: connection_name.clone(),
                        engine_type,
                        status: ConnectionStatus::Connected {
                            dsn: dsn.clone(),
                            masked: masked.clone(),
                        },
                        masked_dsn: masked.clone(),
                    });
                }

                self.explorer.add_source(DbSource {
                    name: connection_name.clone(),
                    engine_type,
                    status: ConnectionStatus::Connected {
                        dsn: dsn.clone(),
                        masked: masked.clone(),
                    },
                    masked_dsn: masked.clone(),
                    tree: Vec::new(),
                    expanded: false,
                });

                self.active_connection = Some(connection_name.clone());

                let output = format!("[{}] Connected to {}", timestamp(), masked);
                self.output_results.output.push(output);

                if let (Some(password), Some(profile_name)) =
                    (self.pending_keychain_password.take(), self.pending_profile_name.take())
                {
                    let profile =
                        self.config.connections.iter().find(|p| p.name == profile_name).cloned();
                    if let Some(profile) = profile {
                        if let Err(e) = profile.store_password(&password) {
                            self.output_results.output.push(format!(
                                "[{}] WARNING: Failed to store password in keychain: {}",
                                timestamp(),
                                e
                            ));
                        } else {
                            self.config.mark_profile_keychain(&profile_name);
                            self.output_results.output.push(format!(
                                "[{}] Password stored in keychain for profile '{}'",
                                timestamp(),
                                profile_name
                            ));
                        }
                    }
                }
                if let Some(tx) = self.db_tx.clone() {
                    let _ =
                        tx.send(DbCommand::LoadSchema { connection_name: connection_name.clone() });
                }
                if let Some(ref runtime) = self.lua_runtime
                    && let Ok(data) = runtime.lua.create_table()
                {
                    let _ = data.set("dsn", masked);
                    runtime.fire_event("ConnectionOpened", data);
                }
            },
            DbEvent::ConnectionFailed { connection_name, message } => {
                if let Some(entry) = self.connection_by_name_mut(connection_name) {
                    entry.status = ConnectionStatus::Error(message.clone());
                } else {
                    self.connections.push(ConnectionEntry {
                        name: connection_name.clone(),
                        engine_type: EngineType::Postgres,
                        status: ConnectionStatus::Error(message.clone()),
                        masked_dsn: String::new(),
                    });
                }
                self.explorer
                    .set_source_status(connection_name, ConnectionStatus::Error(message.clone()));
                let output = format!("[{}] Connection failed: {}", timestamp(), message);
                self.output_results.output.push(output);
            },
            DbEvent::Disconnected { connection_name } => {
                if let Some(entry) = self.connection_by_name_mut(connection_name) {
                    entry.status = ConnectionStatus::Disconnected;
                }
                self.explorer.remove_source(connection_name);
                if self.active_connection.as_deref() == Some(connection_name) {
                    self.active_connection = self
                        .connections
                        .iter()
                        .find(|c| matches!(c.status, ConnectionStatus::Connected { .. }))
                        .map(|c| c.name.clone());
                }
                if let Some(ref runtime) = self.lua_runtime
                    && let Ok(data) = runtime.lua.create_table()
                {
                    runtime.fire_event("ConnectionClosed", data);
                }
            },
            DbEvent::SchemaLoaded { connection_name, nodes } => {
                self.explorer.set_tree_for_source(connection_name, nodes.clone());
            },
            DbEvent::ColumnsLoaded { connection_name, schema, table, columns } => {
                let column_nodes: Vec<SchemaNode> = columns
                    .iter()
                    .map(|c| SchemaNode::Column {
                        name: c.name.clone(),
                        data_type: c.data_type.clone(),
                        nullable: c.nullable,
                        is_primary_key: c.is_primary_key,
                    })
                    .collect();
                self.explorer.insert_columns(connection_name, schema, table, column_nodes);
            },
            DbEvent::QueryStarted { .. } => {
                let is_page_fetch = {
                    let editor = self.focused_editor();
                    editor.auto_paginate && editor.current_page > 0
                };
                if !is_page_fetch {
                    self.create_result_tab();
                    let output = format!("[{}] Query started", timestamp());
                    self.output_results.output.push(output);
                } else {
                    let grid = self.active_result_grid_mut();
                    grid.rows_before_fetch = grid.total_rows_received;
                }
                self.active_result_grid_mut().is_streaming = true;
                self.last_query_error = None;
            },
            DbEvent::ResultColumns { columns, .. } => {
                self.active_result_grid_mut().set_columns(columns.clone());
            },
            DbEvent::QueryRow { cells, .. } => {
                self.active_result_grid_mut().add_row(cells.clone());
            },
            DbEvent::QueryCompleted { _rows_affected, _duration_ms, .. } => {
                self.active_result_grid_mut().is_streaming = false;
                let (is_auto, has_sql, page_size, sql) = {
                    let editor = self.focused_editor();
                    (
                        editor.auto_paginate,
                        editor.last_executed_sql.is_some(),
                        editor.page_size,
                        editor.last_executed_sql.clone(),
                    )
                };
                self.focused_editor_mut().mark_completed();
                let total_rows = self.active_result_grid().total_rows_received;
                let output = format!(
                    "[{}] Query completed: {} rows, {}ms",
                    timestamp(),
                    total_rows,
                    _duration_ms
                );
                self.output_results.output.push(output);
                if is_auto && has_sql {
                    let grid = self.active_result_grid_mut();
                    let rows_this_page =
                        grid.total_rows_received.saturating_sub(grid.rows_before_fetch);
                    if rows_this_page > page_size && !grid.rows.is_empty() {
                        grid.rows.pop_back();
                        grid.total_rows_received = grid.total_rows_received.saturating_sub(1);
                        grid.has_more = true;
                    } else {
                        grid.has_more = false;
                    }
                } else {
                    self.active_result_grid_mut().has_more = false;
                }
                if let Some(ref new_value) = self.cell_edit_new_value.clone() {
                    let row = self.cell_edit_row;
                    let col = self.cell_edit_col;
                    let grid = self.active_result_grid_mut();
                    if let Some(cells) = grid.rows.get_mut(row)
                        && col < cells.len()
                    {
                        cells[col] = new_value.clone();
                    }
                    grid.cancel_edit();
                    self.status_message = Some("Cell updated".into());
                    self.cell_edit_new_value = None;
                }
                if let Some(ref runtime) = self.lua_runtime
                    && let Ok(data) = runtime.lua.create_table()
                {
                    if let Some(ref sql) = sql {
                        let _ = data.set("sql", sql.as_str());
                    }
                    let _ = data.set("rows_affected", *_rows_affected);
                    let _ = data.set("duration_ms", *_duration_ms);
                    runtime.fire_event("QueryExecuted", data);
                }
            },
            DbEvent::QueryError { message, .. } => {
                self.last_query_error = Some(message.clone());
                self.focused_editor_mut().mark_completed();
                let output = format!("[{}] ERROR: {}", timestamp(), message);
                self.output_results.output.push(output);
                if self.cell_edit_new_value.is_some() {
                    self.status_message = Some(format!("UPDATE failed: {}", message));
                    self.cell_edit_new_value = None;
                }
            },
            DbEvent::QueryCancelled { .. } => {
                self.focused_editor_mut().mark_completed();
            },
            DbEvent::TableInfoLoaded { ddl, row_count, table_size, .. } => {
                if let PopupState::QuickDoc {
                    ddl: ddl_dest,
                    row_count: rc_dest,
                    table_size: ts_dest,
                    loading,
                    ..
                } = &mut self.popup
                {
                    *loading = false;
                    *ddl_dest = ddl.clone();
                    *rc_dest = *row_count;
                    *ts_dest = table_size.clone();
                }
            },
        }
    }
}
