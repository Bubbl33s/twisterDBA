use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyEvent};
use secrecy::SecretString;

use crate::db::backend::EngineType;
use crate::db::client::DbCommand;
use crate::editor::tree::TsParser;
use crate::state::{AppState, ConnectForm, ConnectionStatus, Mode};

fn timestamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn engine_from_dsn(dsn: &str) -> EngineType {
    if dsn.starts_with("mysql://") {
        EngineType::Mysql
    } else if dsn.starts_with("sqlite://") || dsn.starts_with("sqlite:") {
        EngineType::Sqlite
    } else {
        EngineType::Postgres
    }
}

pub fn mask_raw_dsn(dsn: &str) -> String {
    if let Some(at_pos) = dsn.find('@') {
        if let Some(scheme_end) = dsn.find("://") {
            let userinfo = &dsn[scheme_end + 3..at_pos];
            if let Some(colon) = userinfo.find(':') {
                let user = &userinfo[..colon];
                let rest = &dsn[at_pos..];
                format!("{}://{}:***{}", &dsn[..scheme_end], user, rest)
            } else {
                dsn.to_string()
            }
        } else {
            format!("***@{}", &dsn[at_pos + 1..])
        }
    } else {
        dsn.to_string()
    }
}

fn collect_keyword_nodes(
    node: &tree_sitter::Node,
    source: &str,
    toggles: &mut Vec<(usize, usize, String)>,
    upper: bool,
) {
    if node.kind() == "keyword" || node.kind().starts_with("keyword_") {
        let text = &source[node.start_byte()..node.end_byte()];
        let toggled = if upper { text.to_uppercase() } else { text.to_lowercase() };
        if toggled != text {
            toggles.push((node.start_byte(), node.end_byte(), toggled));
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_keyword_nodes(&child, source, toggles, upper);
        }
    }
}

impl AppState {
    pub(crate) fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let buffer = match &self.mode {
                    Mode::Command { buffer } => buffer.clone(),
                    _ => return,
                };
                if !buffer.trim().is_empty() {
                    if self.command_history.is_empty()
                        || self.command_history.back().map(|s| s.as_str()) != Some(buffer.trim())
                    {
                        self.command_history.push_back(buffer.trim().to_string());
                        if self.command_history.len() > 100 {
                            self.command_history.pop_front();
                        }
                    }
                    self.history_index = None;
                }
                self.execute_command(buffer);
            },
            KeyCode::Tab => {
                if let Mode::Command { buffer } = &mut self.mode {
                    let partial = buffer.trim();
                    let matches: Vec<&str> =
                        Self::COMMANDS.iter().filter(|c| c.starts_with(partial)).copied().collect();
                    if matches.len() == 1 {
                        *buffer = matches[0].to_string();
                    } else if matches.len() > 1 {
                        let mut prefix = String::from(partial);
                        loop {
                            let next = matches[0].chars().nth(prefix.len());
                            if let Some(next_char) = next {
                                if matches
                                    .iter()
                                    .all(|c| c.chars().nth(prefix.len()) == Some(next_char))
                                {
                                    prefix.push(next_char);
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        if prefix.len() > partial.len() {
                            *buffer = prefix;
                        }
                    }
                }
            },
            KeyCode::Up => {
                if let Mode::Command { buffer } = &mut self.mode {
                    if self.command_history.is_empty() {
                        return;
                    }
                    let idx = match self.history_index {
                        None => self.command_history.len().saturating_sub(1),
                        Some(i) if i > 0 => i - 1,
                        _ => return,
                    };
                    if let Some(entry) = self.command_history.get(idx) {
                        *buffer = entry.clone();
                        self.history_index = Some(idx);
                    }
                }
            },
            KeyCode::Down => {
                if let Mode::Command { buffer } = &mut self.mode {
                    let idx = match self.history_index {
                        Some(i) if i + 1 < self.command_history.len() => i + 1,
                        Some(_) => {
                            self.history_index = None;
                            buffer.clear();
                            return;
                        },
                        None => return,
                    };
                    if let Some(entry) = self.command_history.get(idx) {
                        *buffer = entry.clone();
                        self.history_index = Some(idx);
                    }
                }
            },
            KeyCode::Char(c) => {
                if let Mode::Command { buffer } = &mut self.mode {
                    buffer.push(c);
                }
            },
            KeyCode::Backspace => {
                if let Mode::Command { buffer } = &mut self.mode {
                    buffer.pop();
                }
            },
            _ => {},
        }
    }

    pub const COMMANDS: &[&str] = &[
        "connect",
        "disconnect",
        "quit",
        "export csv",
        "upper",
        "lower",
        "format",
        "help",
        "h",
        "keychain delete",
    ];

    fn toggle_keyword_case(&mut self, upper: bool) {
        let editor = self.focused_editor_mut();
        let source = editor.buffer.get_content();
        if source.is_empty() {
            return;
        }

        let mut parser = TsParser::new();
        let tree = match parser.parse(&source) {
            Some(t) => t,
            None => {
                self.status_message = Some("Failed to parse SQL".into());
                return;
            },
        };

        let root = tree.root_node();
        let mut toggles: Vec<(usize, usize, String)> = Vec::new();
        collect_keyword_nodes(&root, &source, &mut toggles, upper);

        toggles.reverse();

        let mut modified = source.clone();
        for (start, end, replacement) in &toggles {
            modified.replace_range(*start..*end, replacement);
        }

        editor.buffer.set_content(&modified);
        self.status_message =
            Some(if upper { "Keywords uppercased".into() } else { "Keywords lowercased".into() });
    }

    fn execute_command(&mut self, buffer: String) {
        let trimmed = buffer.trim();
        match trimmed {
            "q" | "quit" => self.should_quit = true,
            "help" | "h" => {
                let mut help_lines = vec![
                    "Available commands:".to_string(),
                    "  connect [dsn|profile] - Connect to a database".to_string(),
                    "  disconnect      - Disconnect from current database".to_string(),
                    "  quit | q        - Quit the application".to_string(),
                    "  export csv <path> - Export results to CSV".to_string(),
                    "  upper           - Convert SQL keywords to upper case".to_string(),
                    "  lower           - Convert SQL keywords to lower case".to_string(),
                    "  keychain delete <profile> - Delete stored keychain password".to_string(),
                    "  format          - Format SQL (coming soon)".to_string(),
                    "  lua <code>      - Execute Lua code".to_string(),
                    "  help | h        - Show this help".to_string(),
                ];

                if let Some(ref runtime) = self.lua_runtime {
                    let registries = runtime.registries.borrow();
                    if !registries.commands.is_empty() {
                        help_lines.push(String::new());
                        help_lines.push("Lua-registered commands:".to_string());
                        let mut names: Vec<String> = registries.commands.keys().cloned().collect();
                        names.sort();
                        for name in names {
                            help_lines.push(format!("  :{name}"));
                        }
                    }
                }

                help_lines.push(String::new());
                help_lines.push(
                    "Press : to enter command mode; use Tab to complete; Up/Down for history."
                        .to_string(),
                );

                for line in help_lines {
                    self.output_results.output.push(line);
                }
                self.mode = Mode::Normal;
                return;
            },
            "upper" => {
                self.toggle_keyword_case(true);
                self.mode = Mode::Normal;
                return;
            },
            "lower" => {
                self.toggle_keyword_case(false);
                self.mode = Mode::Normal;
                return;
            },
            "connect" => {
                self.mode = Mode::ConnectDialog { form: ConnectForm::default() };
                return;
            },
            "disconnect" => {
                if let Some(ref conn_name) = self.active_connection.clone()
                    && let Some(tx) = self.db_tx.clone()
                {
                    let _ = tx.send(DbCommand::Disconnect { connection_name: conn_name.clone() });
                }
            },
            cmd if cmd.starts_with("connect ") => {
                let arg = cmd[8..].trim().to_string();
                let is_dsn =
                    arg.contains("://") || arg.starts_with("postgres") || arg.starts_with("mysql");
                if is_dsn {
                    let masked = mask_raw_dsn(&arg);
                    let engine_type = engine_from_dsn(&arg);
                    let connection_name = arg.clone();
                    if let Some(entry) =
                        self.connections.iter_mut().find(|c| c.name == connection_name)
                    {
                        entry.status = ConnectionStatus::Connecting {
                            dsn: arg.clone(),
                            masked: masked.clone(),
                        };
                        entry.masked_dsn = masked.clone();
                    } else {
                        self.connections.push(crate::state::ConnectionEntry {
                            name: connection_name.clone(),
                            engine_type,
                            status: ConnectionStatus::Connecting {
                                dsn: arg.clone(),
                                masked: masked.clone(),
                            },
                            masked_dsn: masked.clone(),
                        });
                    }
                    if let Some(tx) = self.db_tx.clone() {
                        let _ = tx.send(DbCommand::Connect {
                            connection_name,
                            dsn: SecretString::from(arg),
                            engine_type,
                        });
                    }
                } else if let Some(profile) =
                    self.config.connections.iter().find(|p| p.name == arg).cloned()
                {
                    let form = ConnectForm::from_profile(&profile);
                    self.mode = Mode::ConnectDialog { form };
                    return;
                } else {
                    let profile_names: String = {
                        let names: Vec<String> =
                            self.config.connections.iter().map(|p| p.name.clone()).collect();
                        names.join(", ")
                    };
                    self.output_results.output.push(format!(
                        "Profile '{}' not found. Saved profiles: {}",
                        arg, profile_names
                    ));
                }
            },
            cmd if cmd.starts_with("keychain delete ") => {
                let profile_name = cmd[17..].trim().to_string();
                let profile =
                    self.config.connections.iter().find(|p| p.name == profile_name).cloned();
                if let Some(profile) = profile {
                    match profile.delete_password() {
                        Ok(()) => {
                            self.config.clear_profile_keychain(&profile_name);
                            self.output_results.output.push(format!(
                                "Password deleted from keychain for profile '{}'",
                                profile_name
                            ));
                        },
                        Err(e) => {
                            self.output_results
                                .output
                                .push(format!("Failed to delete keychain password: {e}"));
                        },
                    }
                } else {
                    self.output_results
                        .output
                        .push(format!("Profile '{}' not found", profile_name));
                }
            },
            cmd if cmd.starts_with("export csv ") => {
                let path = cmd[11..].trim();
                if let Err(e) = self.export_csv(path) {
                    self.last_query_error = Some(format!("Export failed: {e}"));
                }
            },
            cmd if cmd.starts_with("export ") => {
                let rest = cmd[7..].trim();
                if let Some((format_name, path)) = rest.split_once(' ') {
                    let path = path.trim();
                    if format_name == "csv" {
                        if let Err(e) = self.export_csv(path) {
                            self.last_query_error = Some(format!("Export failed: {e}"));
                        }
                    } else if let Some(ref runtime) = self.lua_runtime {
                        let columns: Vec<String> = self
                            .active_result_grid()
                            .columns
                            .iter()
                            .map(|c| c.name.clone())
                            .collect();
                        let rows: Vec<Vec<String>> = self.active_result_grid().rows.clone().into();
                        match runtime.call_extractor(format_name, columns, rows) {
                            Ok(output) => {
                                if let Err(e) = std::fs::write(path, &output) {
                                    self.last_query_error = Some(format!("Export failed: {e}"));
                                } else {
                                    self.output_results.output.push(format!(
                                        "[{}] Exported to {}",
                                        timestamp(),
                                        path
                                    ));
                                }
                            },
                            Err(e) => {
                                self.last_query_error = Some(format!("Export failed: {e}"));
                            },
                        }
                    } else {
                        self.output_results
                            .output
                            .push(format!("Unknown export format: {}", format_name));
                    }
                }
            },
            cmd if cmd.starts_with("lua ") => {
                let code = cmd[4..].trim();
                match &self.lua_runtime {
                    Some(runtime) => match runtime.execute(code) {
                        Ok(result) => {
                            self.output_results.output.push(format!("Lua: {}", result));
                        },
                        Err(e) => {
                            self.output_results.output.push(format!("Lua error: {}", e));
                        },
                    },
                    None => {
                        self.output_results.output.push("Lua runtime not available".to_string());
                    },
                }
            },
            _ => {
                let func_opt = self.lua_runtime.as_ref().and_then(|runtime| {
                    let registries = runtime.registries.borrow();
                    registries.commands.get(trimmed).cloned()
                });
                if let Some(func) = func_opt {
                    if let Err(e) = func.call::<()>(()) {
                        self.output_results.output.push(format!("Lua command error: {}", e));
                    }
                } else {
                    self.output_results.output.push(format!(
                        "Unknown command: {}. Type :help for available commands.",
                        trimmed
                    ));
                }
            },
        }
        self.mode = Mode::Normal;
    }

    fn export_csv(&self, path: &str) -> anyhow::Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;
        let headers: Vec<String> =
            self.active_result_grid().columns.iter().map(|c| c.name.clone()).collect();
        wtr.write_record(&headers)?;
        for row in &self.active_result_grid().rows {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
