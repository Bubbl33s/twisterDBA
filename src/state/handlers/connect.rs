use crossterm::event::{KeyCode, KeyEvent};
use secrecy::SecretString;

use crate::db::client::DbCommand;
use crate::state::{ConnectionStatus, Mode};

impl super::super::AppState {
    pub(crate) fn handle_connect_dialog_key(&mut self, key: KeyEvent) {
        if let Mode::ConnectDialog { form } = &mut self.mode {
            let field_count = if form.db_type == 2 { 1 } else { 5 };

            match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                },
                KeyCode::Enter => {
                    if form.selecting_type {
                        form.selecting_type = false;
                        return;
                    }
                    let dsn = form.build_dsn();
                    let masked = form.masked_dsn();
                    let engine_type = form.engine_type();
                    let db_type_str = match form.db_type {
                        1 => "mysql",
                        2 => "sqlite",
                        _ => "postgres",
                    };
                    let host = form.fields[0].value.clone();
                    let port_str = if form.fields.len() > 1 { &form.fields[1].value } else { "" };
                    let port: u16 = port_str.parse().unwrap_or(0);
                    let database = if form.fields.len() > 2 {
                        form.fields[2].value.clone()
                    } else {
                        String::new()
                    };
                    let user = if form.fields.len() > 3 {
                        form.fields[3].value.clone()
                    } else {
                        String::new()
                    };
                    let password = if form.fields.len() > 4 {
                        form.fields[4].value.clone()
                    } else {
                        String::new()
                    };
                    let profile_name = format!("{}-{}", db_type_str, &host);
                    self.config.add_profile(crate::config::ConnectionProfile {
                        name: profile_name.clone(),
                        db_type: db_type_str.to_string(),
                        host,
                        port,
                        database,
                        user,
                        password: None,
                    });
                    if !password.is_empty() {
                        self.pending_keychain_password = Some(password);
                        self.pending_profile_name = Some(profile_name);
                    }
                    self.connection_status =
                        ConnectionStatus::Connecting { dsn: dsn.clone(), masked };
                    if let Some(tx) = self.db_tx.clone() {
                        let _ = tx
                            .send(DbCommand::Connect { dsn: SecretString::from(dsn), engine_type });
                    }
                    self.mode = Mode::Normal;
                },
                KeyCode::Tab => {
                    if form.selecting_type {
                        form.selecting_type = false;
                    } else {
                        form.active_field = (form.active_field + 1) % field_count;
                        form.fields[form.active_field].cursor =
                            form.fields[form.active_field].value.len();
                    }
                },
                KeyCode::Down => {
                    if form.selecting_type {
                        form.selecting_type = false;
                    } else {
                        form.active_field = (form.active_field + 1) % field_count;
                        form.fields[form.active_field].cursor =
                            form.fields[form.active_field].value.len();
                    }
                },
                KeyCode::Up => {
                    if !form.selecting_type {
                        if form.active_field == 0 {
                            form.selecting_type = true;
                        } else {
                            form.active_field -= 1;
                            form.fields[form.active_field].cursor =
                                form.fields[form.active_field].value.len();
                        }
                    }
                },
                KeyCode::BackTab => {
                    if !form.selecting_type {
                        if form.active_field == 0 {
                            form.selecting_type = true;
                        } else {
                            form.active_field -= 1;
                            form.fields[form.active_field].cursor =
                                form.fields[form.active_field].value.len();
                        }
                    }
                },
                KeyCode::Left => {
                    if form.selecting_type {
                        form.db_type = match form.db_type {
                            0 => 2,
                            n => n - 1,
                        };
                        if form.active_field >= field_count {
                            form.active_field = field_count.saturating_sub(1);
                            form.fields[form.active_field].cursor =
                                form.fields[form.active_field].value.len();
                        }
                    } else {
                        let field = &mut form.fields[form.active_field];
                        if field.cursor > 0 {
                            field.cursor -= 1;
                        }
                    }
                },
                KeyCode::Right => {
                    if form.selecting_type {
                        form.db_type = (form.db_type + 1) % 3;
                        if form.active_field >= field_count {
                            form.active_field = field_count.saturating_sub(1);
                            form.fields[form.active_field].cursor =
                                form.fields[form.active_field].value.len();
                        }
                    } else {
                        let field = &mut form.fields[form.active_field];
                        if field.cursor < field.value.len() {
                            field.cursor += 1;
                        }
                    }
                },
                KeyCode::Backspace => {
                    if form.selecting_type {
                        return;
                    }
                    let field = &mut form.fields[form.active_field];
                    if field.cursor > 0 {
                        field.cursor -= 1;
                        let idx = field.cursor;
                        if idx < field.value.len() {
                            field.value.remove(idx);
                        }
                    }
                },
                KeyCode::Delete => {
                    if form.selecting_type {
                        return;
                    }
                    let field = &mut form.fields[form.active_field];
                    let idx = field.cursor;
                    if idx < field.value.len() {
                        field.value.remove(idx);
                    }
                },
                KeyCode::Home => {
                    if form.selecting_type {
                        return;
                    }
                    form.fields[form.active_field].cursor = 0;
                },
                KeyCode::End => {
                    if form.selecting_type {
                        return;
                    }
                    let idx = form.fields[form.active_field].value.len();
                    form.fields[form.active_field].cursor = idx;
                },
                KeyCode::Char(c) => {
                    if form.selecting_type {
                        return;
                    }
                    let field = &mut form.fields[form.active_field];
                    field.value.insert(field.cursor, c);
                    field.cursor += 1;
                },
                _ => {},
            }
        }
    }
}
