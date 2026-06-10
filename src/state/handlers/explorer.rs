use crossterm::event::{KeyCode, KeyEvent};

use crate::db::client::DbCommand;
use crate::explorer::NodeKind;

impl super::super::AppState {
    pub(crate) fn handle_explorer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('K') => self.open_quick_doc(),
            KeyCode::Char('j') => self.explorer.select_next(),
            KeyCode::Down => self.explorer.select_next(),
            KeyCode::Char('k') => self.explorer.select_prev(),
            KeyCode::Up => self.explorer.select_prev(),
            KeyCode::Char('h') => {
                let idx = self.explorer.selected_idx;
                self.explorer.collapse_node(idx);
            },
            KeyCode::Char('o') | KeyCode::Char('l') | KeyCode::Enter => {
                self.explorer_toggle_expand();
            },
            KeyCode::Char('R') => {
                if let Some(ref conn_name) = self.active_connection.clone()
                    && let Some(tx) = self.db_tx.clone()
                {
                    let _ = tx.send(DbCommand::LoadSchema { connection_name: conn_name.clone() });
                }
            },
            KeyCode::Esc => {
                self.explorer.clear_search();
            },
            KeyCode::Backspace => {
                self.explorer.pop_search_char();
            },
            KeyCode::Char(c) if c.is_alphanumeric() || c == '_' || c == '.' => {
                self.explorer.push_search_char(c);
            },
            _ => {},
        }
    }

    fn explorer_toggle_expand(&mut self) {
        let idx = self.explorer.selected_idx;
        let kind = self.explorer.node_kind_at(idx);
        match kind {
            Some(NodeKind::Source) => {
                let source_name = self.explorer.node_source_name_at(idx);
                if let Some(name) = source_name {
                    if self.explorer.node_expanded_at(idx) {
                        self.explorer.collapse_source(&name);
                    } else {
                        self.explorer.expand_source(&name);
                        self.active_connection = Some(name.clone());
                        let needs_load =
                            self.explorer.source(&name).is_some_and(|s| s.tree.is_empty());
                        if needs_load && let Some(tx) = self.db_tx.clone() {
                            let _ = tx.send(DbCommand::LoadSchema { connection_name: name });
                        }
                    }
                }
            },
            Some(NodeKind::Schema) => {
                if self.explorer.node_expanded_at(idx) {
                    self.explorer.collapse_node(idx);
                } else {
                    self.explorer.expand_node(idx);
                }
            },
            Some(NodeKind::Table) => {
                if self.explorer.node_loaded_at(idx) {
                    if self.explorer.node_expanded_at(idx) {
                        self.explorer.collapse_node(idx);
                    } else {
                        self.explorer.expand_node(idx);
                    }
                } else {
                    let schema = self.explorer.node_schema_at(idx);
                    let table = self.explorer.node_table_at(idx);
                    if let (Some(schema), Some(table)) = (schema, table) {
                        self.explorer.expand_node(idx);
                        if let Some(ref conn_name) = self.active_connection.clone() {
                            self.explorer.set_loading_child(conn_name, &schema, &table);
                            if let Some(tx) = self.db_tx.clone() {
                                let _ = tx.send(DbCommand::LoadColumns {
                                    connection_name: conn_name.clone(),
                                    schema: schema.clone(),
                                    table: table.clone(),
                                });
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }
}
