use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::warn;

use crate::config::Config;
use crate::db::backend::EngineType;
use crate::db::client::DbCommand;
use crate::editor::SqlEditor;
use crate::editor::tree::TsParser;
use crate::events::DbEvent;
use crate::explorer::{NodeKind, SchemaExplorer, SchemaNode};
use crate::lua::LuaRuntime;
use crate::result::ResultGrid;
use crate::theme::Theme;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command { buffer: String },
    ConnectDialog { form: ConnectForm },
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    SchemaExplorer,
    QueryEditor,
    ResultGrid,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting { dsn: String, masked: String },
    Connected { dsn: String, masked: String },
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectField {
    pub label: &'static str,
    pub value: String,
    pub cursor: usize,
    pub masked: bool,
    pub keychain_loaded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectForm {
    pub fields: Vec<ConnectField>,
    pub active_field: usize,
    pub db_type: usize,
    pub selecting_type: bool,
    pub selected_profile: Option<usize>,
}

impl ConnectForm {
    pub fn default() -> Self {
        Self {
            fields: vec![
                ConnectField {
                    label: "Host",
                    value: "localhost".into(),
                    cursor: 9,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Port",
                    value: "5432".into(),
                    cursor: 4,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Database",
                    value: String::new(),
                    cursor: 0,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "User",
                    value: String::new(),
                    cursor: 0,
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Password",
                    value: String::new(),
                    cursor: 0,
                    masked: true,
                    keychain_loaded: false,
                },
            ],
            active_field: 0,
            db_type: 0,
            selecting_type: true,
            selected_profile: None,
        }
    }

    pub fn from_profile(profile: &crate::config::ConnectionProfile) -> Self {
        let db_type = match profile.db_type.as_str() {
            "mysql" => 1,
            "sqlite" => 2,
            _ => 0,
        };
        let (password_value, keychain_loaded) = if profile.password_is_keychain() {
            match profile.get_password() {
                Ok(pass) => (pass, true),
                Err(_) => (String::new(), false),
            }
        } else {
            (String::new(), false)
        };
        Self {
            fields: vec![
                ConnectField {
                    label: "Host",
                    value: profile.host.clone(),
                    cursor: profile.host.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Port",
                    value: profile.port.to_string(),
                    cursor: profile.port.to_string().len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Database",
                    value: profile.database.clone(),
                    cursor: profile.database.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "User",
                    value: profile.user.clone(),
                    cursor: profile.user.len(),
                    masked: false,
                    keychain_loaded: false,
                },
                ConnectField {
                    label: "Password",
                    value: password_value,
                    cursor: 0,
                    masked: true,
                    keychain_loaded,
                },
            ],
            active_field: 0,
            db_type,
            selecting_type: false,
            selected_profile: None,
        }
    }

    pub fn engine_type(&self) -> EngineType {
        match self.db_type {
            1 => EngineType::Mysql,
            2 => EngineType::Sqlite,
            _ => EngineType::Postgres,
        }
    }

    pub fn build_dsn(&self) -> String {
        match self.db_type {
            1 => self.build_mysql_dsn(),
            2 => self.build_sqlite_dsn(),
            _ => self.build_pg_dsn(),
        }
    }

    fn build_pg_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("postgresql://");

        let has_user = !user.is_empty();
        let has_pass = !pass.is_empty();

        if has_user || has_pass {
            dsn.push_str(user);
            if has_pass {
                dsn.push(':');
                dsn.push_str(pass);
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn build_mysql_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("mysql://");

        let has_user = !user.is_empty();
        let has_pass = !pass.is_empty();

        if has_user || has_pass {
            dsn.push_str(if has_user { user } else { "root" });
            if has_pass {
                dsn.push(':');
                dsn.push_str(pass);
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn build_sqlite_dsn(&self) -> String {
        let path = &self.fields[0].value;
        format!("sqlite://{path}")
    }

    pub fn masked_dsn(&self) -> String {
        match self.db_type {
            1 => self.masked_mysql_dsn(),
            2 => self.masked_sqlite_dsn(),
            _ => self.masked_pg_dsn(),
        }
    }

    fn masked_pg_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("postgresql://");

        let has_auth = !user.is_empty() || !pass.is_empty();
        if has_auth {
            dsn.push_str(user);
            if !pass.is_empty() {
                dsn.push_str(":***");
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn masked_mysql_dsn(&self) -> String {
        let host = &self.fields[0].value;
        let port = &self.fields[1].value;
        let db = &self.fields[2].value;
        let user = &self.fields[3].value;
        let pass = &self.fields[4].value;

        let mut dsn = String::from("mysql://");

        let has_auth = !user.is_empty() || !pass.is_empty();
        if has_auth {
            dsn.push_str(if user.is_empty() { "root" } else { user });
            if !pass.is_empty() {
                dsn.push_str(":***");
            }
            dsn.push('@');
        }

        dsn.push_str(host);
        if !port.is_empty() {
            dsn.push(':');
            dsn.push_str(port);
        }
        if !db.is_empty() {
            dsn.push('/');
            dsn.push_str(db);
        }
        dsn
    }

    fn masked_sqlite_dsn(&self) -> String {
        let path = &self.fields[0].value;
        format!("sqlite://{path}")
    }
}

#[derive(Debug, Clone)]
pub struct OutputPaneState {
    pub lines: VecDeque<String>,
    pub max_lines: usize,
    pub scroll: usize,
}

impl OutputPaneState {
    pub fn new() -> Self {
        Self { lines: VecDeque::new(), max_lines: 500, scroll: 0 }
    }

    pub fn push(&mut self, message: String) {
        self.lines.push_back(message);
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        if self.lines.len() > self.max_lines / 2 {
            self.scroll = self.lines.len().saturating_sub(1);
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll + 1 < self.lines.len() {
            self.scroll += 1;
        }
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        if !self.lines.is_empty() {
            self.scroll = self.lines.len() - 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct CellPopupState {
    pub value: String,
    pub col_name: String,
    pub scroll: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    QuickDoc {
        schema: String,
        table: String,
        ddl: Option<String>,
        row_count: Option<u64>,
        table_size: Option<String>,
        loading: bool,
        scroll: usize,
    },
    KeymapHelp {
        scroll: usize,
    },
}

impl PopupState {
    pub fn is_open(&self) -> bool {
        !matches!(self, PopupState::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSnapshot {
    pub content: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub connection_profile: Option<String>,
    pub buffers: Vec<BufferSnapshot>,
    pub focused_buffer: usize,
    pub focused_panel: String,
    pub split_direction: String,
}

pub struct AppState {
    pub mode: Mode,
    pub focused_panel: Panel,
    pub should_quit: bool,
    pub connection_status: ConnectionStatus,
    pub spinner_frame: usize,
    pub db_tx: Option<mpsc::UnboundedSender<DbCommand>>,
    pub explorer: SchemaExplorer,
    pub editors: Vec<SqlEditor>,
    pub focused_editor: usize,
    pub split_direction: SplitDirection,
    pub result_grid: ResultGrid,
    pub last_query_error: Option<String>,
    pub cell_popup: Option<CellPopupState>,
    pub popup: PopupState,
    pub command_history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub config: Config,
    pub status_message: Option<String>,
    pub theme: Theme,
    pub lua_runtime: Option<LuaRuntime>,
    pending_prefix: Option<char>,
    cell_edit_new_value: Option<String>,
    cell_edit_row: usize,
    cell_edit_col: usize,
    pending_keychain_password: Option<String>,
    pending_profile_name: Option<String>,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_LEN: usize = 10;

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

fn key_matches_config(key: &KeyEvent, binding: &str) -> bool {
    let binding_lower = binding.to_lowercase();
    if let KeyCode::Char(c) = key.code {
        if binding_lower.len() == 5 && binding_lower.starts_with("ctrl+") {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let target = binding_lower.chars().nth(5).unwrap_or('\0');
                return c.to_ascii_lowercase() == target;
            }
            return false;
        }
        if binding_lower.len() == 1 {
            return c.to_ascii_lowercase() == binding_lower.chars().next().unwrap_or('\0');
        }
    }
    false
}

fn key_event_to_string(key: &KeyEvent) -> String {
    match key.code {
        KeyCode::Char(c) => c.to_lowercase().to_string(),
        _ => String::new(),
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            focused_panel: Panel::SchemaExplorer,
            should_quit: false,
            connection_status: ConnectionStatus::Disconnected,
            spinner_frame: 0,
            db_tx: None,
            explorer: SchemaExplorer::new(),
            editors: vec![SqlEditor::new()],
            focused_editor: 0,
            split_direction: SplitDirection::Vertical,
            result_grid: ResultGrid::new(),
            last_query_error: None,
            cell_popup: None,
            popup: PopupState::None,
            command_history: VecDeque::with_capacity(100),
            history_index: None,
            config: Config::load(),
            status_message: None,
            theme: Theme::darcula(),
            lua_runtime: None,
            pending_prefix: None,
            cell_edit_new_value: None,
            cell_edit_row: 0,
            cell_edit_col: 0,
            pending_keychain_password: None,
            pending_profile_name: None,
        }
    }

    pub fn tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_LEN;
    }

    pub fn close_popup(&mut self) {
        self.popup = PopupState::None;
    }

    pub fn focused_editor(&self) -> &SqlEditor {
        &self.editors[self.focused_editor]
    }

    pub fn focused_editor_mut(&mut self) -> &mut SqlEditor {
        &mut self.editors[self.focused_editor]
    }

    pub fn push_editor(&mut self, _direction: SplitDirection) {
        self.editors.push(SqlEditor::new());
        self.focused_editor = self.editors.len() - 1;
    }

    pub fn close_editor(&mut self, index: usize) -> bool {
        if self.editors.len() <= 1 {
            return false;
        }
        self.editors.remove(index);
        if self.focused_editor >= self.editors.len() {
            self.focused_editor = self.editors.len() - 1;
        }
        true
    }

    pub fn focus_editor(&mut self, index: usize) {
        if index < self.editors.len() {
            self.focused_editor = index;
        }
    }

    pub fn spinner_char(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame]
    }

    pub fn to_session_data(&self) -> SessionData {
        let connection_profile = match &self.connection_status {
            ConnectionStatus::Connected { .. } => {
                self.config.connections.iter().find_map(|p| {
                    if p.password_is_keychain() { Some(p.name.clone()) } else { None }
                })
            },
            _ => None,
        };
        let buffers: Vec<BufferSnapshot> = self
            .editors
            .iter()
            .map(|editor| {
                let buffer = &editor.buffer;
                BufferSnapshot {
                    content: buffer.get_content(),
                    cursor_row: buffer.cursor_row,
                    cursor_col: buffer.cursor_col,
                    scroll_offset: buffer.scroll_offset,
                }
            })
            .collect();
        let focused_panel = match self.focused_panel {
            Panel::SchemaExplorer => "schema".to_string(),
            Panel::QueryEditor => "editor".to_string(),
            Panel::ResultGrid => "results".to_string(),
            Panel::Output => "output".to_string(),
        };
        let split_direction = match self.split_direction {
            SplitDirection::Horizontal => "horizontal".to_string(),
            SplitDirection::Vertical => "vertical".to_string(),
        };
        SessionData {
            connection_profile,
            buffers,
            focused_buffer: self.focused_editor,
            focused_panel,
            split_direction,
        }
    }

    pub fn apply_session_data(&mut self, data: SessionData) {
        if !data.buffers.is_empty() {
            self.editors = data
                .buffers
                .iter()
                .map(|snap| {
                    let mut editor = SqlEditor::new();
                    editor.buffer.set_content(&snap.content);
                    editor.buffer.cursor_row = snap.cursor_row;
                    editor.buffer.cursor_col = snap.cursor_col;
                    editor.buffer.scroll_offset = snap.scroll_offset;
                    editor
                })
                .collect();
            self.focused_editor = data.focused_buffer.min(self.editors.len().saturating_sub(1));
        }
        self.focused_panel = match data.focused_panel.as_str() {
            "schema" => Panel::SchemaExplorer,
            "results" => Panel::ResultGrid,
            "output" => Panel::Output,
            _ => Panel::QueryEditor,
        };
        self.split_direction = match data.split_direction.as_str() {
            "horizontal" => SplitDirection::Horizontal,
            _ => SplitDirection::Vertical,
        };
    }

    pub fn apply_db_event(&mut self, event: &DbEvent) {
        match event {
            DbEvent::Connected => {
                let (dsn, masked) = match &self.connection_status {
                    ConnectionStatus::Connecting { dsn, masked } => (dsn.clone(), masked.clone()),
                    other => match other {
                        ConnectionStatus::Connected { dsn, masked } => {
                            (dsn.clone(), masked.clone())
                        },
                        _ => (String::new(), String::new()),
                    },
                };
                let output = format!("[{}] Connected to {}", timestamp(), masked);
                self.focused_editor_mut().output_pane.push(output);
                self.connection_status =
                    ConnectionStatus::Connected { dsn: dsn.clone(), masked: masked.clone() };
                if let (Some(password), Some(profile_name)) =
                    (self.pending_keychain_password.take(), self.pending_profile_name.take())
                {
                    let profile =
                        self.config.connections.iter().find(|p| p.name == profile_name).cloned();
                    if let Some(profile) = profile {
                        if let Err(e) = profile.store_password(&password) {
                            self.focused_editor_mut().output_pane.push(format!(
                                "[{}] WARNING: Failed to store password in keychain: {}",
                                timestamp(),
                                e
                            ));
                        } else {
                            self.config.mark_profile_keychain(&profile_name);
                            self.focused_editor_mut().output_pane.push(format!(
                                "[{}] Password stored in keychain for profile '{}'",
                                timestamp(),
                                profile_name
                            ));
                        }
                    }
                }
                if let Some(tx) = self.db_tx.clone() {
                    let _ = tx.send(DbCommand::LoadSchema);
                }
                if let Some(ref runtime) = self.lua_runtime
                    && let Ok(data) = runtime.lua.create_table()
                {
                    let _ = data.set("dsn", masked);
                    runtime.fire_event("ConnectionOpened", data);
                }
            },
            DbEvent::ConnectionFailed(msg) => {
                self.connection_status = ConnectionStatus::Error(msg.clone());
                let output = format!("[{}] Connection failed: {}", timestamp(), msg);
                self.focused_editor_mut().output_pane.push(output);
            },
            DbEvent::Disconnected => {
                self.connection_status = ConnectionStatus::Disconnected;
                self.explorer = SchemaExplorer::new();
                if let Some(ref runtime) = self.lua_runtime
                    && let Ok(data) = runtime.lua.create_table()
                {
                    runtime.fire_event("ConnectionClosed", data);
                }
            },
            DbEvent::SchemaLoaded(nodes) => {
                self.explorer.set_tree(nodes.clone());
            },
            DbEvent::ColumnsLoaded { schema, table, columns } => {
                let column_nodes: Vec<SchemaNode> = columns
                    .iter()
                    .map(|c| SchemaNode::Column {
                        name: c.name.clone(),
                        data_type: c.data_type.clone(),
                        nullable: c.nullable,
                        is_primary_key: c.is_primary_key,
                    })
                    .collect();
                self.explorer.insert_columns(schema, table, column_nodes);
            },
            DbEvent::QueryStarted => {
                let is_page_fetch = {
                    let editor = self.focused_editor();
                    editor.auto_paginate && editor.current_page > 0
                };
                if !is_page_fetch {
                    self.result_grid.clear();
                    let output = format!("[{}] Query started", timestamp());
                    self.focused_editor_mut().output_pane.push(output);
                } else {
                    self.result_grid.rows_before_fetch = self.result_grid.total_rows_received;
                }
                self.result_grid.is_streaming = true;
                self.last_query_error = None;
            },
            DbEvent::ResultColumns(columns) => {
                self.result_grid.set_columns(columns.clone());
            },
            DbEvent::QueryRow(cells) => {
                self.result_grid.add_row(cells.clone());
            },
            DbEvent::QueryCompleted { _rows_affected, _duration_ms } => {
                self.result_grid.is_streaming = false;
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
                let total_rows = self.result_grid.total_rows_received;
                let output = format!(
                    "[{}] Query completed: {} rows, {}ms",
                    timestamp(),
                    total_rows,
                    _duration_ms
                );
                self.focused_editor_mut().output_pane.push(output);
                if is_auto && has_sql {
                    let rows_this_page = self
                        .result_grid
                        .total_rows_received
                        .saturating_sub(self.result_grid.rows_before_fetch);
                    if rows_this_page > page_size && !self.result_grid.rows.is_empty() {
                        self.result_grid.rows.pop_back();
                        self.result_grid.total_rows_received =
                            self.result_grid.total_rows_received.saturating_sub(1);
                        self.result_grid.has_more = true;
                    } else {
                        self.result_grid.has_more = false;
                    }
                } else {
                    self.result_grid.has_more = false;
                }
                if let Some(ref new_value) = self.cell_edit_new_value.clone() {
                    let row = self.cell_edit_row;
                    let col = self.cell_edit_col;
                    if let Some(cells) = self.result_grid.rows.get_mut(row)
                        && col < cells.len()
                    {
                        cells[col] = new_value.clone();
                    }
                    self.result_grid.cancel_edit();
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
            DbEvent::QueryError(msg) => {
                self.last_query_error = Some(msg.clone());
                self.focused_editor_mut().mark_completed();
                let output = format!("[{}] ERROR: {}", timestamp(), msg);
                self.focused_editor_mut().output_pane.push(output);
                if self.cell_edit_new_value.is_some() {
                    self.status_message = Some(format!("UPDATE failed: {}", msg));
                    self.cell_edit_new_value = None;
                }
            },
            DbEvent::QueryCancelled => {
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Insert => self.handle_insert_key(key),
            Mode::Command { .. } => self.handle_command_key(key),
            Mode::ConnectDialog { .. } => self.handle_connect_dialog_key(key),
            Mode::Visual => self.handle_visual_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
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
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            if self.focused_panel == Panel::SchemaExplorer {
                self.open_quick_doc();
            }
            return;
        }
        match key.code {
            KeyCode::Char('K') => {
                if self.focused_panel == Panel::SchemaExplorer {
                    self.open_quick_doc();
                }
                return;
            },
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                return;
            },
            KeyCode::Char(':') => {
                self.mode = Mode::Command { buffer: String::new() };
                return;
            },
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            },
            KeyCode::Char('v') => {
                self.mode = Mode::Visual;
                return;
            },
            KeyCode::Char('?') => {
                self.popup = PopupState::KeymapHelp { scroll: 0 };
                return;
            },
            _ => {},
        }
        match self.focused_panel {
            Panel::SchemaExplorer => self.handle_explorer_key(key),
            Panel::QueryEditor => self.handle_editor_normal_key(key),
            Panel::ResultGrid => self.handle_result_normal_key(key),
            Panel::Output => self.handle_output_key(key),
        }
    }

    fn handle_editor_normal_key(&mut self, key: KeyEvent) {
        let execute_binding = self.config.keybindings.get("execute_query");
        let cancel_binding = self.config.keybindings.get("cancel_query");

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('e') => {
                    if let Some(tx) = self.db_tx.clone()
                        && !self.focused_editor_mut().execute(&tx)
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
                    && !self.focused_editor_mut().execute(&tx)
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
                self.mode = Mode::Insert;
            },
            KeyCode::Char('o') => {
                self.focused_editor_mut().buffer.insert_newline_below();
                self.mode = Mode::Insert;
            },
            KeyCode::Char('O') => {
                self.focused_editor_mut().buffer.insert_newline_above();
                self.mode = Mode::Insert;
            },
            KeyCode::Tab => {
                self.focused_panel = Panel::ResultGrid;
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
                if !self.close_editor(self.focused_editor) {
                    self.status_message = Some("Cannot close last buffer".into());
                }
            },
            KeyCode::Char('h') => {
                if self.focused_editor > 0 {
                    self.focus_editor(self.focused_editor - 1);
                }
            },
            KeyCode::Char('j') => {
                if self.focused_editor > 0 {
                    self.focus_editor(self.focused_editor - 1);
                }
            },
            KeyCode::Char('l') => {
                if self.focused_editor + 1 < self.editors.len() {
                    self.focus_editor(self.focused_editor + 1);
                }
            },
            KeyCode::Char('k') => {
                if self.focused_editor > 0 {
                    self.focus_editor(self.focused_editor - 1);
                }
            },
            KeyCode::Char('=') => {},
            _ => {},
        }
    }

    fn handle_result_normal_key(&mut self, key: KeyEvent) {
        if self.result_grid.is_editing() {
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
                    self.result_grid.page_down();
                    return;
                },
                KeyCode::Char('u') => {
                    self.result_grid.page_up();
                    return;
                },
                _ => {},
            }
        }

        if self.result_grid.visual_mode {
            match key.code {
                KeyCode::Esc => {
                    self.result_grid.visual_mode = false;
                    self.result_grid.visual_start = None;
                },
                KeyCode::Char('j') | KeyCode::Down => {
                    self.result_grid.move_selection(1, 0);
                    self.check_auto_paginate();
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    self.result_grid.move_selection(-1, 0);
                },
                KeyCode::Char('h') | KeyCode::Left => {
                    self.result_grid.move_selection(0, -1);
                },
                KeyCode::Char('l') | KeyCode::Right => {
                    self.result_grid.move_selection(0, 1);
                },
                KeyCode::Char('y') => {
                    if let Some(tsv) = self.result_grid.visual_range_tsv() {
                        let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(tsv));
                    }
                    self.result_grid.visual_mode = false;
                    self.result_grid.visual_start = None;
                },
                _ => {},
            }
            return;
        }

        match key.code {
            KeyCode::Char('g') => {
                if self.result_grid.last_key == Some('g') {
                    self.result_grid.first_row();
                    self.result_grid.last_key = None;
                } else {
                    self.result_grid.last_key = Some('g');
                }
            },
            other => {
                self.result_grid.last_key = None;
                match other {
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.result_grid.move_selection(1, 0);
                        self.check_auto_paginate();
                    },
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.result_grid.move_selection(-1, 0);
                    },
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.result_grid.move_selection(0, -1);
                    },
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.result_grid.move_selection(0, 1);
                    },
                    KeyCode::Char('G') => {
                        self.result_grid.last_row();
                        self.check_auto_paginate();
                    },
                    KeyCode::Char('H') => {
                        self.result_grid.first_col();
                    },
                    KeyCode::Char('L') => {
                        self.result_grid.last_col();
                    },
                    KeyCode::Enter => {
                        if let Some(value) = self.result_grid.selected_cell_value() {
                            let col_name = self
                                .result_grid
                                .columns
                                .get(self.result_grid.selected_col)
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
                        if let Some(value) = self.result_grid.selected_cell_value() {
                            let _ = arboard::Clipboard::new()
                                .and_then(|mut c| c.set_text(value.to_string()));
                        }
                    },
                    KeyCode::Char('Y') => {
                        let tsv = self.result_grid.selected_row_tsv();
                        if !tsv.is_empty() {
                            let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(tsv));
                        }
                    },
                    KeyCode::Char('v') => {
                        self.result_grid.visual_mode = true;
                        self.result_grid.visual_start =
                            Some((self.result_grid.selected_row, self.result_grid.selected_col));
                    },
                    KeyCode::Char('e') => {
                        let selected_col = self.result_grid.selected_col;
                        let is_pk = selected_col < self.result_grid.columns.len()
                            && self.result_grid.columns[selected_col].is_primary_key;
                        if is_pk {
                            self.status_message = Some("Cannot edit primary key column".into());
                        } else {
                            let value_opt =
                                self.result_grid.selected_cell_value().map(|s| s.to_string());
                            let row = self.result_grid.selected_row;
                            let col = self.result_grid.selected_col;
                            if let Some(value) = value_opt {
                                self.result_grid.enter_edit(row, col, &value);
                            } else {
                                self.status_message = Some(
                                    "Cannot edit: no primary key available for this result set"
                                        .into(),
                                );
                            }
                        }
                    },
                    KeyCode::Tab => {
                        self.focused_panel = Panel::Output;
                    },
                    _ => {},
                }
            },
        }
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
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
            KeyCode::Char('l') | KeyCode::Enter => {
                let idx = self.explorer.selected_idx;
                let kind = self.explorer.node_kind_at(idx);

                match kind {
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
                                self.explorer.set_loading_child(&schema, &table);
                                if let Some(tx) = self.db_tx.clone() {
                                    let _ = tx.send(DbCommand::LoadColumns {
                                        schema: schema.clone(),
                                        table: table.clone(),
                                    });
                                }
                            }
                        }
                    },
                    _ => {},
                }
            },
            KeyCode::Tab => {
                self.focused_panel = Panel::QueryEditor;
            },
            KeyCode::Char('R') => {
                if let Some(tx) = self.db_tx.clone() {
                    let _ = tx.send(DbCommand::LoadSchema);
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

    fn handle_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.focused_panel = Panel::SchemaExplorer;
            },
            KeyCode::Char('j') => {
                self.focused_editor_mut().output_pane.scroll_down();
            },
            KeyCode::Char('k') => {
                self.focused_editor_mut().output_pane.scroll_up();
            },
            KeyCode::Char('g') => {
                self.focused_editor_mut().output_pane.scroll_top();
            },
            KeyCode::Char('G') => {
                self.focused_editor_mut().output_pane.scroll_bottom();
            },
            _ => {},
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = Mode::Normal;
            return;
        }
        if self.focused_panel == Panel::QueryEditor {
            let execute_binding = self.config.keybindings.get("execute_query");
            let cancel_binding = self.config.keybindings.get("cancel_query");

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
                if let Some(tx) = self.db_tx.clone()
                    && !self.focused_editor_mut().execute(&tx)
                {
                    self.status_message = Some("No statement under cursor".into());
                }
                return;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                self.focused_editor_mut().cancel_query();
                return;
            }
            if let Some(binding) = execute_binding
                && key_matches_config(&key, binding)
            {
                if let Some(tx) = self.db_tx.clone()
                    && !self.focused_editor_mut().execute(&tx)
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
            self.focused_editor_mut().handle_insert_key(&key);
        }
    }

    const COMMANDS: &[&str] = &[
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

    fn handle_command_key(&mut self, key: KeyEvent) {
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

    fn handle_visual_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = Mode::Normal;
        }
    }

    fn handle_connect_dialog_key(&mut self, key: KeyEvent) {
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

    fn handle_popup_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close_popup();
            },
            KeyCode::Char('j') => match &mut self.popup {
                PopupState::QuickDoc { scroll, .. } => *scroll = scroll.saturating_add(1),
                PopupState::KeymapHelp { scroll } => *scroll = scroll.saturating_add(1),
                _ => {},
            },
            KeyCode::Char('k') => match &mut self.popup {
                PopupState::QuickDoc { scroll, .. } => *scroll = scroll.saturating_sub(1),
                PopupState::KeymapHelp { scroll } => *scroll = scroll.saturating_sub(1),
                _ => {},
            },
            KeyCode::Char('g') => match &mut self.popup {
                PopupState::QuickDoc { scroll, .. } => *scroll = 0,
                PopupState::KeymapHelp { scroll } => *scroll = 0,
                _ => {},
            },
            KeyCode::Char('G') => match &mut self.popup {
                PopupState::QuickDoc { scroll, .. } => *scroll = usize::MAX,
                PopupState::KeymapHelp { scroll } => *scroll = usize::MAX,
                _ => {},
            },
            KeyCode::Char('?') => {
                self.close_popup();
            },
            _ => {},
        }
    }

    fn open_quick_doc(&mut self) {
        let idx = self.explorer.selected_idx;
        let kind = self.explorer.node_kind_at(idx);
        if !matches!(kind, Some(NodeKind::Table) | Some(NodeKind::View)) {
            self.status_message = Some("Quick Doc available for tables and views only".into());
            return;
        }
        let schema = self.explorer.node_schema_at(idx);
        let table = self.explorer.node_table_at(idx);
        if let (Some(schema), Some(table)) = (schema, table) {
            self.popup = PopupState::QuickDoc {
                schema: schema.clone(),
                table: table.clone(),
                ddl: None,
                row_count: None,
                table_size: None,
                loading: true,
                scroll: 0,
            };
            if let Some(tx) = self.db_tx.clone() {
                let _ = tx.send(DbCommand::LoadTableInfo {
                    schema: schema.clone(),
                    table: table.clone(),
                });
            }
        }
    }

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
                    self.focused_editor_mut().output_pane.push(line);
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
                if let Some(tx) = self.db_tx.clone() {
                    let _ = tx.send(DbCommand::Disconnect);
                }
            },
            cmd if cmd.starts_with("connect ") => {
                let arg = cmd[8..].trim().to_string();
                let is_dsn =
                    arg.contains("://") || arg.starts_with("postgres") || arg.starts_with("mysql");
                if is_dsn {
                    let masked = mask_raw_dsn(&arg);
                    let engine_type = engine_from_dsn(&arg);
                    self.connection_status =
                        ConnectionStatus::Connecting { dsn: arg.clone(), masked };
                    if let Some(tx) = self.db_tx.clone() {
                        let _ = tx
                            .send(DbCommand::Connect { dsn: SecretString::from(arg), engine_type });
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
                    self.focused_editor_mut().output_pane.push(format!(
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
                            self.focused_editor_mut().output_pane.push(format!(
                                "Password deleted from keychain for profile '{}'",
                                profile_name
                            ));
                        },
                        Err(e) => {
                            self.focused_editor_mut()
                                .output_pane
                                .push(format!("Failed to delete keychain password: {e}"));
                        },
                    }
                } else {
                    self.focused_editor_mut()
                        .output_pane
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
                        let columns: Vec<String> =
                            self.result_grid.columns.iter().map(|c| c.name.clone()).collect();
                        let rows: Vec<Vec<String>> = self.result_grid.rows.clone().into();
                        match runtime.call_extractor(format_name, columns, rows) {
                            Ok(output) => {
                                if let Err(e) = std::fs::write(path, &output) {
                                    self.last_query_error = Some(format!("Export failed: {e}"));
                                } else {
                                    self.focused_editor_mut().output_pane.push(format!(
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
                        self.focused_editor_mut()
                            .output_pane
                            .push(format!("Unknown export format: {}", format_name));
                    }
                }
            },
            cmd if cmd.starts_with("lua ") => {
                let code = cmd[4..].trim();
                match &self.lua_runtime {
                    Some(runtime) => match runtime.execute(code) {
                        Ok(result) => {
                            self.focused_editor_mut().output_pane.push(format!("Lua: {}", result));
                        },
                        Err(e) => {
                            self.focused_editor_mut().output_pane.push(format!("Lua error: {}", e));
                        },
                    },
                    None => {
                        self.focused_editor_mut()
                            .output_pane
                            .push("Lua runtime not available".to_string());
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
                        self.focused_editor_mut()
                            .output_pane
                            .push(format!("Lua command error: {}", e));
                    }
                } else {
                    self.focused_editor_mut().output_pane.push(format!(
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
            self.result_grid.columns.iter().map(|c| c.name.clone()).collect();
        wtr.write_record(&headers)?;
        for row in &self.result_grid.rows {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn check_auto_paginate(&mut self) {
        if !self.result_grid.needs_next_page() {
            return;
        }
        let next_page = self.focused_editor().current_page + 1;
        if let Some(tx) = self.db_tx.clone() {
            self.focused_editor_mut().fetch_next_page(&tx, next_page);
        }
    }

    fn toggle_auto_paginate(&mut self) {
        let editor = self.focused_editor_mut();
        editor.auto_paginate = !editor.auto_paginate;
        self.status_message =
            Some(format!("Auto-pagination: {}", if editor.auto_paginate { "on" } else { "off" }));
    }

    fn handle_edit_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.result_grid.cancel_edit();
            },
            KeyCode::Enter => {
                self.commit_cell_edit();
            },
            KeyCode::Backspace => {
                self.result_grid.edit_delete_backward();
            },
            KeyCode::Left => {
                self.result_grid.edit_move_cursor(false);
            },
            KeyCode::Right => {
                self.result_grid.edit_move_cursor(true);
            },
            KeyCode::Char(c) => {
                self.result_grid.edit_insert_char(c);
            },
            _ => {},
        }
    }

    fn commit_cell_edit(&mut self) {
        let (_old_value, new_value) = match self.result_grid.commit_edit() {
            Some(v) => v,
            None => return,
        };
        let row_idx = self.result_grid.selected_row;
        let selected_col = self.result_grid.selected_col;
        let col_name =
            self.result_grid.columns.get(selected_col).map(|c| c.name.clone()).unwrap_or_default();
        let pk_clauses = self.result_grid.pk_where_clause(row_idx, &self.result_grid.columns);

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
                let schema = self.result_grid.source_schema.clone().unwrap_or_default();
                let table = self.result_grid.source_table.clone().unwrap_or_default();
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

                if let Some(tx) = self.db_tx.clone() {
                    let cancel = tokio_util::sync::CancellationToken::new();
                    let _ = tx.send(DbCommand::ExecuteQuery {
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
                self.result_grid.cancel_edit();
            },
        }
    }
}

fn session_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("twisterDBA")
}

fn session_path() -> PathBuf {
    session_dir().join("session.toml")
}

pub fn save_session_to_disk(data: &SessionData) -> Result<(), String> {
    let dir = session_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create session dir: {e}"))?;
    let path = session_path();
    let toml_str =
        toml::to_string_pretty(data).map_err(|e| format!("Failed to serialize session: {e}"))?;
    fs::write(&path, toml_str).map_err(|e| format!("Failed to write session file: {e}"))?;
    Ok(())
}

pub fn load_session_from_disk() -> Result<Option<SessionData>, String> {
    let path = session_path();
    if !path.exists() {
        return Ok(None);
    }
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<SessionData>(&content) {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                warn!("Corrupt session file at {:?}: {}. Starting fresh.", path, e);
                let _ = fs::remove_file(&path);
                Ok(None)
            },
        },
        Err(e) => {
            warn!("Failed to read session file at {:?}: {}. Starting fresh.", path, e);
            Ok(None)
        },
    }
}
