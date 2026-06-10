use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::editor::SqlEditor;
use crate::state::{AppState, EditorSplit, SplitDirection, Window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSnapshot {
    pub content: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub active_connection: Option<String>,
    pub saved_connections: Vec<String>,
    pub buffers: Vec<BufferSnapshot>,
    pub focused_buffer: usize,
    pub focused_panel: String,
    pub split_direction: String,
}

impl AppState {
    pub fn to_session_data(&self) -> SessionData {
        let saved_connections: Vec<String> = self
            .connections
            .iter()
            .filter(|c| matches!(c.status, super::connection::ConnectionStatus::Connected { .. }))
            .map(|c| c.name.clone())
            .collect();
        let buffers: Vec<BufferSnapshot> = self
            .editor_splits
            .iter()
            .flat_map(|split| split.tabs.iter())
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
        let focused_window = match self.focused_window {
            Window::SchemaExplorer => "schema".to_string(),
            Window::QueryEditor => "editor".to_string(),
            Window::OutputResults => "output".to_string(),
        };
        let split_direction = match self.split_direction {
            SplitDirection::Horizontal => "horizontal".to_string(),
            SplitDirection::Vertical => "vertical".to_string(),
        };
        SessionData {
            active_connection: self.active_connection.clone(),
            saved_connections,
            buffers,
            focused_buffer: self.active_split,
            focused_panel: focused_window,
            split_direction,
        }
    }

    pub fn apply_session_data(&mut self, data: SessionData) {
        if !data.buffers.is_empty() {
            let editors: Vec<SqlEditor> = data
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
            self.editor_splits = vec![EditorSplit { tabs: editors, active_tab: 0 }];
            self.active_split = data.focused_buffer.min(self.editor_splits.len().saturating_sub(1));
        }
        self.focused_window = match data.focused_panel.as_str() {
            "schema" => Window::SchemaExplorer,
            "output" => Window::OutputResults,
            _ => Window::QueryEditor,
        };
        self.split_direction = match data.split_direction.as_str() {
            "horizontal" => SplitDirection::Horizontal,
            _ => SplitDirection::Vertical,
        };
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
