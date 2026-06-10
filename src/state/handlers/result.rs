use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::db::client::DbCommand;
use crate::state::CellPopupState;

impl super::super::AppState {
    pub(crate) fn handle_result_normal_key(&mut self, key: KeyEvent) {
        if self.active_result_grid_mut().is_editing() {
            self.handle_edit_mode(key);
            return;
        }

        if self.cell_popup.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                    self.cell_popup = None;
                },
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some(ref mut popup) = self.cell_popup {
                        popup.scroll = popup.scroll.saturating_add(1);
                    }
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    if let Some(ref mut popup) = self.cell_popup {
                        popup.scroll = popup.scroll.saturating_sub(1);
                    }
                },
                _ => {},
            }
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('d') => {
                    self.active_result_grid_mut().page_down();
                    return;
                },
                KeyCode::Char('u') => {
                    self.active_result_grid_mut().page_up();
                    return;
                },
                _ => {},
            }
        }

        if self.active_result_grid_mut().visual_mode {
            match key.code {
                KeyCode::Esc => {
                    self.active_result_grid_mut().visual_mode = false;
                    self.active_result_grid_mut().visual_start = None;
                },
                KeyCode::Char('j') | KeyCode::Down => {
                    self.active_result_grid_mut().move_selection(1, 0);
                    self.check_auto_paginate();
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    self.active_result_grid_mut().move_selection(-1, 0);
                },
                KeyCode::Char('h') | KeyCode::Left => {
                    self.active_result_grid_mut().move_selection(0, -1);
                },
                KeyCode::Char('l') | KeyCode::Right => {
                    self.active_result_grid_mut().move_selection(0, 1);
                },
                KeyCode::Char('y') => {
                    if let Some(tsv) = self.active_result_grid_mut().visual_range_tsv() {
                        let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(tsv));
                    }
                    self.active_result_grid_mut().visual_mode = false;
                    self.active_result_grid_mut().visual_start = None;
                },
                _ => {},
            }
            return;
        }

        match key.code {
            KeyCode::Char('g') => {
                if self.active_result_grid_mut().last_key == Some('g') {
                    self.active_result_grid_mut().first_row();
                    self.active_result_grid_mut().last_key = None;
                } else {
                    self.active_result_grid_mut().last_key = Some('g');
                }
            },
            other => {
                self.active_result_grid_mut().last_key = None;
                match other {
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.active_result_grid_mut().move_selection(1, 0);
                        self.check_auto_paginate();
                    },
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.active_result_grid_mut().move_selection(-1, 0);
                    },
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.active_result_grid_mut().move_selection(0, -1);
                    },
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.active_result_grid_mut().move_selection(0, 1);
                    },
                    KeyCode::Char('G') => {
                        self.active_result_grid_mut().last_row();
                        self.check_auto_paginate();
                    },
                    KeyCode::Char('H') => {
                        self.active_result_grid_mut().first_col();
                    },
                    KeyCode::Char('L') => {
                        self.active_result_grid_mut().last_col();
                    },
                    KeyCode::Enter => {
                        if let Some(value) = self.active_result_grid().selected_cell_value() {
                            let col_name = self
                                .active_result_grid()
                                .columns
                                .get(self.active_result_grid().selected_col)
                                .map(|c| c.name.clone())
                                .unwrap_or_default();
                            self.cell_popup = Some(CellPopupState {
                                value: value.to_string(),
                                col_name,
                                scroll: 0,
                            });
                        }
                    },
                    KeyCode::Char('y') => {
                        if let Some(value) = self.active_result_grid_mut().selected_cell_value() {
                            let _ = arboard::Clipboard::new()
                                .and_then(|mut c| c.set_text(value.to_string()));
                        }
                    },
                    KeyCode::Char('Y') => {
                        let tsv = self.active_result_grid_mut().selected_row_tsv();
                        if !tsv.is_empty() {
                            let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(tsv));
                        }
                    },
                    KeyCode::Char('v') => {
                        self.active_result_grid_mut().visual_mode = true;
                        self.active_result_grid_mut().visual_start = Some((
                            self.active_result_grid_mut().selected_row,
                            self.active_result_grid_mut().selected_col,
                        ));
                    },
                    KeyCode::Char('e') => {
                        let selected_col = self.active_result_grid_mut().selected_col;
                        let is_pk = selected_col < self.active_result_grid_mut().columns.len()
                            && self.active_result_grid_mut().columns[selected_col].is_primary_key;
                        if is_pk {
                            self.status_message = Some("Cannot edit primary key column".into());
                        } else {
                            let value_opt = self
                                .active_result_grid_mut()
                                .selected_cell_value()
                                .map(|s| s.to_string());
                            let row = self.active_result_grid_mut().selected_row;
                            let col = self.active_result_grid_mut().selected_col;
                            if let Some(value) = value_opt {
                                self.active_result_grid_mut().enter_edit(row, col, &value);
                            } else {
                                self.status_message = Some(
                                    "Cannot edit: no primary key available for this result set"
                                        .into(),
                                );
                            }
                        }
                    },
                    KeyCode::Tab => {
                        self.output_results.next_tab();
                    },
                    KeyCode::BackTab => {
                        self.output_results.prev_tab();
                    },
                    _ => {},
                }
            },
        }
    }

    fn handle_edit_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.active_result_grid_mut().cancel_edit();
            },
            KeyCode::Enter => {
                self.commit_cell_edit();
            },
            KeyCode::Backspace => {
                self.active_result_grid_mut().edit_delete_backward();
            },
            KeyCode::Left => {
                self.active_result_grid_mut().edit_move_cursor(false);
            },
            KeyCode::Right => {
                self.active_result_grid_mut().edit_move_cursor(true);
            },
            KeyCode::Char(c) => {
                self.active_result_grid_mut().edit_insert_char(c);
            },
            _ => {},
        }
    }

    fn commit_cell_edit(&mut self) {
        let commit_result = self
            .active_result_grid_mut()
            .commit_edit()
            .map(|(o, n)| (o.to_string(), n.to_string()));
        let (_old_value, new_value) = match commit_result {
            Some(v) => v,
            None => return,
        };
        let row_idx = self.active_result_grid().selected_row;
        let selected_col = self.active_result_grid().selected_col;
        let col_name = self
            .active_result_grid()
            .columns
            .get(selected_col)
            .map(|c| c.name.clone())
            .unwrap_or_default();
        let pk_clauses = {
            let grid = self.active_result_grid();
            grid.pk_where_clause(row_idx, &grid.columns)
        };
        let schema = self.active_result_grid().source_schema.clone().unwrap_or_default();
        let table = self.active_result_grid().source_table.clone().unwrap_or_default();

        match pk_clauses {
            Some(ref clauses) if !clauses.is_empty() => {
                let where_str = clauses
                    .iter()
                    .map(|(name, val)| format!("{} = {}", name, val))
                    .collect::<Vec<_>>()
                    .join(" AND ");
                let set_value = if new_value.is_empty() {
                    "NULL".to_string()
                } else {
                    format!("'{}'", new_value.replace('\'', "''"))
                };
                let update_sql = if schema.is_empty() || table.is_empty() {
                    format!("UPDATE UNKNOWN SET {} = {} WHERE {}", col_name, set_value, where_str)
                } else {
                    format!(
                        "UPDATE {}.{} SET {} = {} WHERE {}",
                        schema, table, col_name, set_value, where_str
                    )
                };

                self.cell_edit_new_value = Some(new_value.to_string());
                self.cell_edit_row = row_idx;
                self.cell_edit_col = selected_col;

                if let Some(tx) = self.db_tx.clone()
                    && let Some(ref conn_name) = self.active_connection.clone()
                {
                    let cancel = tokio_util::sync::CancellationToken::new();
                    let _ = tx.send(DbCommand::ExecuteQuery {
                        connection_name: conn_name.clone(),
                        sql: update_sql,
                        cancel,
                        auto_paginate: false,
                        page_size: 0,
                    });
                }
            },
            _ => {
                self.status_message =
                    Some("Cannot edit: no primary key available for this result set".into());
                self.active_result_grid_mut().cancel_edit();
            },
        }
    }

    pub(crate) fn check_auto_paginate(&mut self) {
        if !self.active_result_grid_mut().needs_next_page() {
            return;
        }
        let next_page = self.focused_editor().current_page + 1;
        if let Some(tx) = self.db_tx.clone()
            && let Some(ref conn_name) = self.active_connection.clone()
        {
            self.focused_editor_mut().fetch_next_page(&tx, conn_name, next_page);
        }
    }

    pub(crate) fn toggle_auto_paginate(&mut self) {
        let editor = self.focused_editor_mut();
        editor.auto_paginate = !editor.auto_paginate;
        self.status_message =
            Some(format!("Auto-pagination: {}", if editor.auto_paginate { "on" } else { "off" }));
    }
}
