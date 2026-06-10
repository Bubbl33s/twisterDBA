use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::warn;

use super::key_event_to_string;
use super::key_matches_config;
use crate::editor::Direction;
use crate::state::{PopupState, SplitDirection, Window};

impl super::super::AppState {
    pub(crate) fn handle_normal_key(&mut self, key: KeyEvent) {
        if let Some(ref runtime) = self.lua_runtime {
            let mode_str = "n";
            let key_str = key_event_to_string(&key);
            if !key_str.is_empty() {
                let lookup = format!("{}|{}", mode_str, key_str);
                let registries = runtime.registries.borrow();
                if let Some(func) = registries.keymaps.get(&lookup) {
                    if let Err(e) = func.call::<()>(()) {
                        warn!("Lua keymap callback error: {}", e);
                    }
                    return;
                }
            }
        }
        if self.cell_popup.is_some() {
            self.handle_result_normal_key(key);
            return;
        }

        if self.popup.is_open() {
            self.handle_popup_key(key);
            return;
        }

        if let Some(prefix) = self.pending_prefix.take()
            && prefix == 'w'
        {
            self.handle_window_command(key);
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
            self.pending_prefix = Some('w');
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && key.modifiers.contains(KeyModifiers::SHIFT)
        {
            match key.code {
                KeyCode::Char('h') => {
                    if self.focused_window == Window::QueryEditor {
                        self.move_tab_left();
                    }
                    return;
                },
                KeyCode::Char('l') => {
                    if self.focused_window == Window::QueryEditor {
                        self.move_tab_right();
                    }
                    return;
                },
                _ => {},
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('h') => {
                    self.navigate_window(Direction::Left);
                    return;
                },
                KeyCode::Char('j') => {
                    self.navigate_window(Direction::Down);
                    return;
                },
                KeyCode::Char('k') => {
                    self.navigate_window(Direction::Up);
                    return;
                },
                KeyCode::Char('l') => {
                    self.navigate_window(Direction::Right);
                    return;
                },
                KeyCode::Char('q') => {
                    if self.focused_window == Window::SchemaExplorer {
                        self.open_quick_doc();
                    }
                    return;
                },
                _ => {},
            }
        }
        match key.code {
            KeyCode::Char('K') => {
                if self.focused_window == Window::SchemaExplorer {
                    self.open_quick_doc();
                }
                return;
            },
            KeyCode::Char('i') => {
                self.mode = super::super::Mode::Insert;
                return;
            },
            KeyCode::Char(':') => {
                self.mode = super::super::Mode::Command { buffer: String::new() };
                return;
            },
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            },
            KeyCode::Char('v') => {
                self.mode = super::super::Mode::Visual;
                return;
            },
            KeyCode::Char('?') => {
                self.popup = PopupState::KeymapHelp { scroll: 0 };
                return;
            },
            _ => {},
        }

        if self.pending_space {
            self.pending_space = false;
            match key.code {
                KeyCode::Char('b') => {
                    if self.focused_window == Window::QueryEditor {
                        self.push_tab();
                    }
                    return;
                },
                KeyCode::Char('x') => {
                    match self.focused_window {
                        Window::QueryEditor => {
                            if !self.close_tab() {
                                self.status_message = Some("Cannot close last tab".into());
                            }
                        },
                        Window::OutputResults if self.output_results.active_tab > 0 => {
                            let idx = self.output_results.active_tab - 1;
                            if !self.close_result_tab(idx) {
                                self.status_message = Some("Cannot close last result tab".into());
                            }
                        },
                        _ => {},
                    }
                    return;
                },
                _ => {},
            }
        }

        if let KeyCode::Char(' ') = key.code {
            self.pending_space = true;
            return;
        }

        match self.focused_window {
            Window::SchemaExplorer => self.handle_explorer_key(key),
            Window::QueryEditor => self.handle_editor_normal_key(key),
            Window::OutputResults => self.handle_output_results_key(key),
        }
    }

    fn handle_editor_normal_key(&mut self, key: KeyEvent) {
        let execute_binding = self.config.keybindings.get("execute_query");
        let cancel_binding = self.config.keybindings.get("cancel_query");

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('e') => {
                    if let Some(tx) = self.db_tx.clone()
                        && let Some(ref conn_name) = self.active_connection.clone()
                        && !self.focused_editor_mut().execute(&tx, conn_name)
                    {
                        self.status_message = Some("No statement under cursor".into());
                    }
                    return;
                },
                KeyCode::Char('c') => {
                    self.focused_editor_mut().cancel_query();
                    return;
                },
                KeyCode::Char('p') => {
                    let current = self.focused_editor_mut().buffer.get_content();
                    if let Some(sql) = self.focused_editor_mut().history.navigate_previous(&current)
                    {
                        self.focused_editor_mut().buffer.set_content(&sql);
                    }
                    return;
                },
                KeyCode::Char('n') => {
                    if let Some(sql) = self.focused_editor_mut().history.navigate_next() {
                        self.focused_editor_mut().buffer.set_content(&sql);
                    }
                    return;
                },
                KeyCode::Char('t') => {
                    self.toggle_auto_paginate();
                    return;
                },
                _ => {},
            }

            if let Some(binding) = execute_binding
                && key_matches_config(&key, binding)
            {
                if let Some(tx) = self.db_tx.clone()
                    && let Some(ref conn_name) = self.active_connection.clone()
                    && !self.focused_editor_mut().execute(&tx, conn_name)
                {
                    self.status_message = Some("No statement under cursor".into());
                }
                return;
            }
            if let Some(binding) = cancel_binding
                && key_matches_config(&key, binding)
            {
                self.focused_editor_mut().cancel_query();
                return;
            }
        }

        match key.code {
            KeyCode::Char('A') => {
                self.focused_editor_mut().buffer.line_end();
                self.mode = super::super::Mode::Insert;
            },
            KeyCode::Char('o') => {
                self.focused_editor_mut().buffer.insert_newline_below();
                self.mode = super::super::Mode::Insert;
            },
            KeyCode::Char('O') => {
                self.focused_editor_mut().buffer.insert_newline_above();
                self.mode = super::super::Mode::Insert;
            },
            KeyCode::Tab => {
                self.next_tab();
            },
            KeyCode::BackTab => {
                self.prev_tab();
            },
            _ => {
                self.focused_editor_mut().handle_normal_key(&key);
            },
        }
    }

    fn handle_window_command(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') => {
                self.push_editor(SplitDirection::Horizontal);
            },
            KeyCode::Char('v') => {
                self.push_editor(SplitDirection::Vertical);
            },
            KeyCode::Char('q') => {
                if !self.close_editor(self.active_split) {
                    self.status_message = Some("Cannot close last buffer".into());
                }
            },
            KeyCode::Char('o') => {
                self.close_other_editors(self.active_split);
            },
            KeyCode::Char('w') => {
                self.cycle_windows();
            },
            KeyCode::Char('h') => {
                if self.focused_window == Window::QueryEditor && self.active_split > 0 {
                    self.focus_editor(self.active_split - 1);
                } else {
                    self.navigate_window(Direction::Left);
                }
            },
            KeyCode::Char('j') => {
                if self.focused_window == Window::QueryEditor
                    && self.split_direction == SplitDirection::Horizontal
                    && self.active_split + 1 < self.editor_splits.len()
                {
                    self.focus_editor(self.active_split + 1);
                } else {
                    self.navigate_window(Direction::Down);
                }
            },
            KeyCode::Char('k') => {
                if self.focused_window == Window::QueryEditor
                    && self.split_direction == SplitDirection::Horizontal
                    && self.active_split > 0
                {
                    self.focus_editor(self.active_split - 1);
                } else {
                    self.navigate_window(Direction::Up);
                }
            },
            KeyCode::Char('l') => {
                if self.focused_window == Window::QueryEditor
                    && self.active_split + 1 < self.editor_splits.len()
                {
                    self.focus_editor(self.active_split + 1);
                } else {
                    self.navigate_window(Direction::Right);
                }
            },
            KeyCode::Char('=') => {},
            _ => {},
        }
    }

    fn handle_output_results_key(&mut self, key: KeyEvent) {
        if self.output_results.active_tab == 0 {
            match key.code {
                KeyCode::Tab => {
                    self.output_results.next_tab();
                },
                KeyCode::BackTab => {
                    self.output_results.prev_tab();
                },
                KeyCode::Char('j') => {
                    self.output_results.output.scroll_down();
                },
                KeyCode::Char('k') => {
                    self.output_results.output.scroll_up();
                },
                KeyCode::Char('g') => {
                    self.output_results.output.scroll_top();
                },
                KeyCode::Char('G') => {
                    self.output_results.output.scroll_bottom();
                },
                _ => {},
            }
        } else {
            self.handle_result_normal_key(key);
        }
    }
}
