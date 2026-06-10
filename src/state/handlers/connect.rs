use crossterm::event::{KeyCode, KeyEvent};
use secrecy::SecretString;

use crate::db::client::DbCommand;
use crate::state::{ConnectionEntry, ConnectionStatus, DialogStep, Mode};

impl super::super::AppState {
    pub(crate) fn handle_connect_dialog_key(&mut self, key: KeyEvent) {
        if let Mode::ConnectDialog { form } = &mut self.mode {
            match form.step {
                DialogStep::SelectType => self.handle_step1_key(key),
                DialogStep::EnterDetails => self.handle_step2_key(key),
            }
        }
    }

    fn handle_step1_key(&mut self, key: KeyEvent) {
        let profile_count = self.config.connections.len();
        if let Mode::ConnectDialog { form } = &mut self.mode {
            match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                },
                KeyCode::Left => {
                    if form.selected_profile.is_none() {
                        form.type_cursor =
                            if form.type_cursor == 0 { 2 } else { form.type_cursor - 1 };
                    }
                },
                KeyCode::Right => {
                    if form.selected_profile.is_none() {
                        form.type_cursor = (form.type_cursor + 1) % 3;
                    }
                },
                KeyCode::Down => {
                    if form.selected_profile.is_none() {
                        if profile_count > 0 {
                            form.selected_profile = Some(0);
                        }
                    } else if let Some(ref mut idx) = form.selected_profile
                        && *idx + 1 < profile_count
                    {
                        *idx += 1;
                    }
                },
                KeyCode::Up => {
                    if let Some(ref mut idx) = form.selected_profile {
                        if *idx == 0 {
                            form.selected_profile = None;
                        } else {
                            *idx -= 1;
                        }
                    }
                },
                KeyCode::Enter => {
                    if let Some(profile_idx) = form.selected_profile {
                        if let Some(profile) = self.config.connections.get(profile_idx).cloned() {
                            let db_type = match profile.db_type.as_str() {
                                "mysql" => 1,
                                "sqlite" => 2,
                                _ => 0,
                            };
                            let (password_value, keychain_loaded) =
                                if profile.password_is_keychain() {
                                    match profile.get_password() {
                                        Ok(pass) => (pass, true),
                                        Err(_) => (String::new(), false),
                                    }
                                } else {
                                    (String::new(), false)
                                };
                            form.db_type = db_type;
                            form.fields = match db_type {
                                2 => vec![crate::state::ConnectField {
                                    label: "File Path",
                                    value: profile.host.clone(),
                                    cursor: profile.host.len(),
                                    masked: false,
                                    keychain_loaded: false,
                                }],
                                _ => vec![
                                    crate::state::ConnectField {
                                        label: "Host",
                                        value: profile.host.clone(),
                                        cursor: profile.host.len(),
                                        masked: false,
                                        keychain_loaded: false,
                                    },
                                    crate::state::ConnectField {
                                        label: "Port",
                                        value: profile.port.to_string(),
                                        cursor: profile.port.to_string().len(),
                                        masked: false,
                                        keychain_loaded: false,
                                    },
                                    crate::state::ConnectField {
                                        label: "Database",
                                        value: profile.database.clone(),
                                        cursor: profile.database.len(),
                                        masked: false,
                                        keychain_loaded: false,
                                    },
                                    crate::state::ConnectField {
                                        label: "User",
                                        value: profile.user.clone(),
                                        cursor: profile.user.len(),
                                        masked: false,
                                        keychain_loaded: false,
                                    },
                                    crate::state::ConnectField {
                                        label: "Password",
                                        value: password_value,
                                        cursor: 0,
                                        masked: true,
                                        keychain_loaded,
                                    },
                                ],
                            };
                            form.connection_name = profile.name.clone();
                            form.connection_name_cursor = profile.name.len();
                            form.ssl_mode = 2;
                            form.type_cursor = db_type;
                            form.active_field = 0;
                            form.name_conflict = false;
                            form.step = DialogStep::EnterDetails;
                        }
                    } else {
                        form.db_type = form.type_cursor;
                        form.fields =
                            crate::state::ConnectForm::fields_for_engine_pub(form.db_type);
                        let name = form.auto_generate_name();
                        form.connection_name_cursor = name.len();
                        form.connection_name = name;
                        form.active_field = 0;
                        form.name_conflict = false;
                        form.step = DialogStep::EnterDetails;
                    }
                },
                _ => {},
            }
        }
    }

    fn handle_step2_key(&mut self, key: KeyEvent) {
        if let Mode::ConnectDialog { form } = &mut self.mode {
            let total_fields = form.total_field_count();

            if form.name_conflict {
                match key.code {
                    KeyCode::Esc => {
                        form.name_conflict = false;
                        return;
                    },
                    KeyCode::Enter => {
                        form.name_conflict = false;
                        self.do_connect();
                        return;
                    },
                    _ => {
                        form.name_conflict = false;
                    },
                }
            }

            match key.code {
                KeyCode::Esc => {
                    form.step = DialogStep::SelectType;
                },
                KeyCode::Enter => {
                    let name = form.connection_name.clone();
                    let name_exists = self.config.connections.iter().any(|p| p.name == name);
                    if name_exists {
                        form.name_conflict = true;
                        return;
                    }
                    self.do_connect();
                },
                KeyCode::Tab | KeyCode::Down => {
                    form.active_field = (form.active_field + 1) % total_fields;
                    Self::reset_cursor_for_field(form);
                },
                KeyCode::BackTab => {
                    form.active_field = if form.active_field == 0 {
                        total_fields - 1
                    } else {
                        form.active_field - 1
                    };
                    Self::reset_cursor_for_field(form);
                },
                KeyCode::Up => {
                    if form.active_field == 0 {
                        form.active_field = total_fields - 1;
                    } else {
                        form.active_field -= 1;
                    }
                    Self::reset_cursor_for_field(form);
                },
                KeyCode::Left => {
                    if form.active_field == 0 {
                        if form.connection_name_cursor > 0 {
                            form.connection_name_cursor -= 1;
                        }
                    } else if Self::is_ssl_mode_field(form) {
                        form.ssl_mode = if form.ssl_mode == 0 {
                            crate::state::ConnectForm::SSL_MODES.len() - 1
                        } else {
                            form.ssl_mode - 1
                        };
                    } else {
                        let field = &mut form.fields[form.active_field - 1];
                        if field.cursor > 0 {
                            field.cursor -= 1;
                        }
                    }
                },
                KeyCode::Right => {
                    if form.active_field == 0 {
                        if form.connection_name_cursor < form.connection_name.len() {
                            form.connection_name_cursor += 1;
                        }
                    } else if Self::is_ssl_mode_field(form) {
                        form.ssl_mode =
                            (form.ssl_mode + 1) % crate::state::ConnectForm::SSL_MODES.len();
                    } else {
                        let field = &mut form.fields[form.active_field - 1];
                        if field.cursor < field.value.len() {
                            field.cursor += 1;
                        }
                    }
                },
                KeyCode::Backspace => {
                    if form.active_field == 0 {
                        if form.connection_name_cursor > 0 {
                            form.connection_name_cursor -= 1;
                            form.connection_name.remove(form.connection_name_cursor);
                        }
                    } else if Self::is_ssl_mode_field(form) {
                    } else {
                        let field = &mut form.fields[form.active_field - 1];
                        if field.cursor > 0 {
                            field.cursor -= 1;
                            let idx = field.cursor;
                            if idx < field.value.len() {
                                field.value.remove(idx);
                            }
                        }
                    }
                },
                KeyCode::Delete => {
                    if form.active_field == 0 {
                        if form.connection_name_cursor < form.connection_name.len() {
                            form.connection_name.remove(form.connection_name_cursor);
                        }
                    } else if Self::is_ssl_mode_field(form) {
                    } else {
                        let field = &mut form.fields[form.active_field - 1];
                        let idx = field.cursor;
                        if idx < field.value.len() {
                            field.value.remove(idx);
                        }
                    }
                },
                KeyCode::Home => {
                    if form.active_field == 0 {
                        form.connection_name_cursor = 0;
                    } else if !Self::is_ssl_mode_field(form) {
                        form.fields[form.active_field - 1].cursor = 0;
                    }
                },
                KeyCode::End => {
                    if form.active_field == 0 {
                        form.connection_name_cursor = form.connection_name.len();
                    } else if !Self::is_ssl_mode_field(form) {
                        let idx = form.fields[form.active_field - 1].value.len();
                        form.fields[form.active_field - 1].cursor = idx;
                    }
                },
                KeyCode::Char(c) => {
                    if form.active_field == 0 {
                        form.connection_name.insert(form.connection_name_cursor, c);
                        form.connection_name_cursor += 1;
                    } else if Self::is_ssl_mode_field(form) {
                    } else {
                        let field = &mut form.fields[form.active_field - 1];
                        field.value.insert(field.cursor, c);
                        field.cursor += 1;
                    }
                },
                _ => {},
            }
        }
    }

    fn is_ssl_mode_field(form: &crate::state::ConnectForm) -> bool {
        form.db_type == 0 && form.active_field == form.fields.len() + 1
    }

    fn reset_cursor_for_field(form: &mut crate::state::ConnectForm) {
        if form.active_field == 0 {
            form.connection_name_cursor = form.connection_name.len();
        } else if form.db_type == 0 && form.active_field == form.fields.len() + 1 {
        } else if form.active_field <= form.fields.len() {
            let idx = form.active_field - 1;
            form.fields[idx].cursor = form.fields[idx].value.len();
        }
    }

    fn do_connect(&mut self) {
        if let Mode::ConnectDialog { form } = &self.mode {
            let dsn = form.build_dsn();
            let masked = form.masked_dsn();
            let engine_type = form.engine_type();
            let db_type_str = match form.db_type {
                1 => "mysql",
                2 => "sqlite",
                _ => "postgres",
            };
            let host = form.fields.first().map(|f| f.value.clone()).unwrap_or_default();
            let port_str = if form.fields.len() > 1 { &form.fields[1].value } else { "" };
            let port: u16 = port_str.parse().unwrap_or(0);
            let database =
                if form.fields.len() > 2 { form.fields[2].value.clone() } else { String::new() };
            let user =
                if form.fields.len() > 3 { form.fields[3].value.clone() } else { String::new() };
            let password =
                if form.fields.len() > 4 { form.fields[4].value.clone() } else { String::new() };
            let profile_name = form.connection_name.clone();
            self.config.upsert_profile(crate::config::ConnectionProfile {
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
                self.pending_profile_name = Some(profile_name.clone());
            }
            let masked_dsn = masked.clone();
            if let Some(entry) = self.connections.iter_mut().find(|c| c.name == profile_name) {
                entry.status =
                    ConnectionStatus::Connecting { dsn: dsn.clone(), masked: masked.clone() };
                entry.masked_dsn = masked_dsn.clone();
            } else {
                self.connections.push(ConnectionEntry {
                    name: profile_name.clone(),
                    engine_type,
                    status: ConnectionStatus::Connecting {
                        dsn: dsn.clone(),
                        masked: masked.clone(),
                    },
                    masked_dsn,
                });
            }
            if let Some(tx) = self.db_tx.clone() {
                let _ = tx.send(DbCommand::Connect {
                    connection_name: profile_name,
                    dsn: SecretString::from(dsn),
                    engine_type,
                });
            }
        }
        self.mode = Mode::Normal;
    }
}
