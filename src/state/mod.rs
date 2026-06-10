pub mod connection;
pub mod events;
pub mod handlers;
pub mod mode;
pub mod output;
pub mod popup;
pub mod session;

use std::collections::VecDeque;

use tokio::sync::mpsc;

use crate::config::Config;
use crate::db::client::DbCommand;
use crate::editor::Direction;
use crate::editor::SqlEditor;
use crate::explorer::{DbSource, SchemaExplorer};
use crate::lua::LuaRuntime;
use crate::result::ResultGrid;
use crate::theme::Theme;

pub use connection::{ConnectField, ConnectForm, ConnectionEntry, ConnectionStatus, DialogStep};
pub use handlers::command::mask_raw_dsn;
pub use mode::{Mode, SplitDirection, Window};
pub use output::{CellPopupState, OutputPaneState, OutputResultsState};
pub use popup::PopupState;
pub use session::{load_session_from_disk, save_session_to_disk};

pub struct EditorSplit {
    pub tabs: Vec<SqlEditor>,
    pub active_tab: usize,
}

impl EditorSplit {
    pub fn new() -> Self {
        Self { tabs: vec![SqlEditor::new()], active_tab: 0 }
    }

    pub fn active_editor(&self) -> &SqlEditor {
        &self.tabs[self.active_tab]
    }

    pub fn active_editor_mut(&mut self) -> &mut SqlEditor {
        &mut self.tabs[self.active_tab]
    }
}

pub struct AppState {
    pub mode: Mode,
    pub focused_window: Window,
    pub should_quit: bool,
    pub connections: Vec<ConnectionEntry>,
    pub active_connection: Option<String>,
    pub spinner_frame: usize,
    pub db_tx: Option<mpsc::UnboundedSender<DbCommand>>,
    pub explorer: SchemaExplorer,
    pub editor_splits: Vec<EditorSplit>,
    pub active_split: usize,
    pub split_direction: SplitDirection,
    pub output_results: OutputResultsState,
    pub last_query_error: Option<String>,
    pub cell_popup: Option<CellPopupState>,
    pub popup: PopupState,
    pub command_history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub config: Config,
    pub status_message: Option<String>,
    pub theme: Theme,
    pub lua_runtime: Option<LuaRuntime>,
    pub(crate) pending_prefix: Option<char>,
    pub(crate) pending_space: bool,
    pub(crate) cell_edit_new_value: Option<String>,
    pub(crate) cell_edit_row: usize,
    pub(crate) cell_edit_col: usize,
    pub(crate) pending_keychain_password: Option<String>,
    pub(crate) pending_profile_name: Option<String>,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_LEN: usize = 10;

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            focused_window: Window::SchemaExplorer,
            should_quit: false,
            connections: Vec::new(),
            active_connection: None,
            spinner_frame: 0,
            db_tx: None,
            explorer: SchemaExplorer::new(),
            editor_splits: vec![EditorSplit::new()],
            active_split: 0,
            split_direction: SplitDirection::Vertical,
            output_results: OutputResultsState::new(),
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
            pending_space: false,
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

    pub fn focused_split(&self) -> &EditorSplit {
        &self.editor_splits[self.active_split]
    }

    pub fn focused_split_mut(&mut self) -> &mut EditorSplit {
        &mut self.editor_splits[self.active_split]
    }

    pub fn focused_editor(&self) -> &SqlEditor {
        self.focused_split().active_editor()
    }

    pub fn focused_editor_mut(&mut self) -> &mut SqlEditor {
        self.focused_split_mut().active_editor_mut()
    }

    pub fn push_editor(&mut self, _direction: SplitDirection) {
        self.editor_splits.push(EditorSplit::new());
        self.active_split = self.editor_splits.len() - 1;
        self.sync_output_to_editor_focus();
    }

    pub fn close_editor(&mut self, index: usize) -> bool {
        if self.editor_splits.len() <= 1 {
            return false;
        }
        let to_close: Vec<(usize, usize)> =
            self.editor_splits[index].tabs.iter().enumerate().map(|(ti, _)| (index, ti)).collect();
        for (si, ti) in &to_close {
            self.remove_linked_results(*si, *ti);
        }
        self.editor_splits.remove(index);
        if self.active_split >= self.editor_splits.len() {
            self.active_split = self.editor_splits.len() - 1;
        }
        self.sync_output_to_editor_focus();
        true
    }

    pub fn close_other_editors(&mut self, index: usize) {
        while self.editor_splits.len() > 1 {
            let close_idx = if index == 0 { 1 } else { 0 };
            self.close_editor(close_idx);
        }
    }

    pub fn focus_editor(&mut self, index: usize) {
        if index < self.editor_splits.len() {
            self.active_split = index;
            self.sync_output_to_editor_focus();
        }
    }

    pub fn navigate_window(&mut self, direction: Direction) {
        if self.focused_window == Window::QueryEditor && self.editor_splits.len() > 1 {
            self.navigate_within_query_editor(direction);
            return;
        }
        if let Some(window) = self.window_target(self.focused_window, direction) {
            if window == Window::QueryEditor && self.editor_splits.len() > 1 {
                self.active_split = match (self.focused_window, direction) {
                    (Window::SchemaExplorer, Direction::Left) => self.editor_splits.len() - 1,
                    (Window::SchemaExplorer, Direction::Right) => 0,
                    (Window::OutputResults, Direction::Up) => {
                        self.active_split.min(self.editor_splits.len() - 1)
                    },
                    _ => self.active_split.min(self.editor_splits.len() - 1),
                };
                self.sync_output_to_editor_focus();
            }
            self.focused_window = window;
        }
    }

    fn navigate_within_query_editor(&mut self, direction: Direction) {
        let n = self.editor_splits.len();
        let cur = self.active_split.min(n - 1);
        match direction {
            Direction::Left => {
                if cur > 0 {
                    self.active_split = cur - 1;
                } else {
                    self.focused_window = Window::SchemaExplorer;
                }
                self.sync_output_to_editor_focus();
            },
            Direction::Right => {
                if cur + 1 < n {
                    self.active_split = cur + 1;
                } else {
                    self.focused_window = Window::OutputResults;
                }
                self.sync_output_to_editor_focus();
            },
            Direction::Up => {
                self.focused_window = Window::SchemaExplorer;
            },
            Direction::Down => {
                self.focused_window = Window::OutputResults;
            },
        }
    }

    fn window_target(&self, from: Window, direction: Direction) -> Option<Window> {
        match (from, direction) {
            (Window::SchemaExplorer, Direction::Left) => Some(Window::QueryEditor),
            (Window::SchemaExplorer, Direction::Right) => Some(Window::QueryEditor),
            (Window::SchemaExplorer, Direction::Up) => Some(Window::OutputResults),
            (Window::SchemaExplorer, Direction::Down) => Some(Window::OutputResults),
            (Window::QueryEditor, Direction::Left) => Some(Window::SchemaExplorer),
            (Window::QueryEditor, Direction::Right) => Some(Window::OutputResults),
            (Window::QueryEditor, Direction::Up) => Some(Window::SchemaExplorer),
            (Window::QueryEditor, Direction::Down) => Some(Window::OutputResults),
            (Window::OutputResults, Direction::Left) => Some(Window::SchemaExplorer),
            (Window::OutputResults, Direction::Right) => Some(Window::SchemaExplorer),
            (Window::OutputResults, Direction::Up) => Some(Window::QueryEditor),
            (Window::OutputResults, Direction::Down) => Some(Window::SchemaExplorer),
        }
    }

    pub fn cycle_windows(&mut self) {
        self.focused_window = match self.focused_window {
            Window::SchemaExplorer => Window::QueryEditor,
            Window::QueryEditor => Window::OutputResults,
            Window::OutputResults => Window::SchemaExplorer,
        };
    }

    pub fn push_tab(&mut self) {
        let split = self.focused_split_mut();
        split.tabs.push(SqlEditor::new());
        split.active_tab = split.tabs.len() - 1;
        self.sync_output_to_editor_focus();
    }

    pub fn close_tab(&mut self) -> bool {
        let tab_idx = self.focused_split().active_tab;
        if self.focused_split().tabs.len() <= 1 {
            return false;
        }
        let split_idx = self.active_split;
        self.remove_linked_results(split_idx, tab_idx);
        let split = self.focused_split_mut();
        split.tabs.remove(tab_idx);
        if split.active_tab >= split.tabs.len() {
            split.active_tab = split.tabs.len() - 1;
        }
        self.sync_output_to_editor_focus();
        true
    }

    pub fn next_tab(&mut self) {
        let split = self.focused_split_mut();
        if !split.tabs.is_empty() {
            split.active_tab = (split.active_tab + 1) % split.tabs.len();
        }
        self.sync_output_to_editor_focus();
    }

    pub fn prev_tab(&mut self) {
        let split = self.focused_split_mut();
        if !split.tabs.is_empty() {
            split.active_tab =
                if split.active_tab == 0 { split.tabs.len() - 1 } else { split.active_tab - 1 };
        }
        self.sync_output_to_editor_focus();
    }

    pub fn move_tab_left(&mut self) {
        let split = self.focused_split_mut();
        if split.active_tab > 0 {
            split.tabs.swap(split.active_tab, split.active_tab - 1);
            split.active_tab -= 1;
        }
    }

    pub fn move_tab_right(&mut self) {
        let split = self.focused_split_mut();
        if split.active_tab + 1 < split.tabs.len() {
            split.tabs.swap(split.active_tab, split.active_tab + 1);
            split.active_tab += 1;
        }
    }

    fn remove_linked_results(&mut self, split_idx: usize, tab_idx: usize) {
        let output = &mut self.output_results;
        let to_remove: Vec<usize> = output
            .result_tabs
            .iter()
            .enumerate()
            .filter(|(_, rt)| rt.source_split == Some(split_idx) && rt.source_tab == Some(tab_idx))
            .map(|(i, _)| i)
            .collect();
        for idx in to_remove.into_iter().rev() {
            output.close_result_tab(idx);
        }
    }

    pub fn active_result_grid(&self) -> &ResultGrid {
        self.output_results.active_result_grid()
    }

    pub fn active_result_grid_mut(&mut self) -> &mut ResultGrid {
        self.output_results.active_result_grid_mut()
    }

    pub fn create_result_tab(&mut self) -> usize {
        let split_idx = self.active_split;
        let tab_idx = self.focused_split().active_tab;
        self.output_results.find_or_create_result_tab(split_idx, tab_idx)
    }

    pub fn close_result_tab(&mut self, index: usize) -> bool {
        self.output_results.close_result_tab(index)
    }

    fn sync_output_to_editor_focus(&mut self) {
        if self.focused_window != Window::QueryEditor {
            return;
        }
        let split_idx = self.active_split;
        let tab_idx = self.focused_split().active_tab;
        if let Some(idx) = self
            .output_results
            .result_tabs
            .iter()
            .position(|t| t.source_split == Some(split_idx) && t.source_tab == Some(tab_idx))
        {
            self.output_results.active_tab = idx + 1;
        }
    }

    pub fn spinner_char(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame]
    }

    pub fn connection_by_name(&self, name: &str) -> Option<&ConnectionEntry> {
        self.connections.iter().find(|c| c.name == name)
    }

    pub fn connection_by_name_mut(&mut self, name: &str) -> Option<&mut ConnectionEntry> {
        self.connections.iter_mut().find(|c| c.name == name)
    }

    pub fn active_source(&self) -> Option<&DbSource> {
        let name = self.active_connection.as_ref()?;
        self.explorer.sources.iter().find(|s| s.name == *name)
    }

    #[allow(dead_code)]
    pub fn set_active_source(&mut self, name: &str) {
        if self.explorer.sources.iter().any(|s| s.name == name) {
            self.active_connection = Some(name.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_creates_independent_result_tabs() {
        let mut state = AppState::new();
        assert_eq!(state.editor_splits.len(), 1);
        assert_eq!(state.active_split, 0);
        assert_eq!(state.output_results.result_tabs.len(), 1);
        assert!(state.output_results.result_tabs[0].source_split.is_none());

        state.push_editor(SplitDirection::Vertical);
        assert_eq!(state.editor_splits.len(), 2);
        assert_eq!(state.active_split, 1);

        state.focus_editor(0);
        state.create_result_tab();
        assert_eq!(state.output_results.result_tabs.len(), 1);
        assert_eq!(state.output_results.result_tabs[0].source_split, Some(0));
        assert_eq!(state.output_results.active_tab, 1);

        state.focus_editor(1);
        state.create_result_tab();
        assert_eq!(state.output_results.result_tabs.len(), 2);
        assert_eq!(state.output_results.result_tabs[0].source_split, Some(0));
        assert_eq!(state.output_results.result_tabs[1].source_split, Some(1));
        assert_eq!(state.output_results.active_tab, 2);
        assert_eq!(state.output_results.result_tabs[1].title, "Result 2");
    }

    #[test]
    fn navigation_cycles_editor_splits() {
        let mut state = AppState::new();
        state.push_editor(SplitDirection::Vertical);
        state.focused_window = Window::QueryEditor;
        state.active_split = 1;

        state.navigate_window(Direction::Left);
        assert_eq!(state.active_split, 0);
        assert_eq!(state.focused_window, Window::QueryEditor);

        state.navigate_window(Direction::Right);
        assert_eq!(state.active_split, 1);
        assert_eq!(state.focused_window, Window::QueryEditor);

        state.navigate_window(Direction::Right);
        assert_eq!(state.focused_window, Window::OutputResults);
    }

    #[test]
    fn navigation_window_wrap_around() {
        let mut state = AppState::new();
        state.focused_window = Window::SchemaExplorer;

        state.navigate_window(Direction::Left);
        assert_eq!(state.focused_window, Window::QueryEditor);
        assert_eq!(state.active_split, 0);

        state.navigate_window(Direction::Left);
        assert_eq!(state.focused_window, Window::SchemaExplorer);

        state.navigate_window(Direction::Right);
        assert_eq!(state.focused_window, Window::QueryEditor);
        assert_eq!(state.active_split, 0);

        state.navigate_window(Direction::Right);
        assert_eq!(state.focused_window, Window::OutputResults);

        state.navigate_window(Direction::Up);
        assert_eq!(state.focused_window, Window::QueryEditor);

        state.navigate_window(Direction::Up);
        assert_eq!(state.focused_window, Window::SchemaExplorer);

        state.navigate_window(Direction::Down);
        assert_eq!(state.focused_window, Window::OutputResults);

        state.navigate_window(Direction::Down);
        assert_eq!(state.focused_window, Window::SchemaExplorer);
    }

    #[test]
    fn navigation_window_with_multiple_editors() {
        let mut state = AppState::new();
        state.push_editor(SplitDirection::Vertical);
        state.push_editor(SplitDirection::Vertical);

        state.focused_window = Window::SchemaExplorer;

        state.navigate_window(Direction::Left);
        assert_eq!(state.focused_window, Window::QueryEditor);
        assert_eq!(state.active_split, 2);

        state.navigate_window(Direction::Left);
        assert_eq!(state.active_split, 1);

        state.navigate_window(Direction::Left);
        assert_eq!(state.active_split, 0);

        state.navigate_window(Direction::Left);
        assert_eq!(state.focused_window, Window::SchemaExplorer);

        state.navigate_window(Direction::Right);
        assert_eq!(state.focused_window, Window::QueryEditor);
        assert_eq!(state.active_split, 0);

        state.navigate_window(Direction::Right);
        assert_eq!(state.active_split, 1);

        state.navigate_window(Direction::Right);
        assert_eq!(state.active_split, 2);

        state.navigate_window(Direction::Right);
        assert_eq!(state.focused_window, Window::OutputResults);

        state.navigate_window(Direction::Up);
        assert_eq!(state.focused_window, Window::QueryEditor);
        assert_eq!(state.active_split, 2);

        state.navigate_window(Direction::Up);
        assert_eq!(state.focused_window, Window::SchemaExplorer);

        state.navigate_window(Direction::Down);
        assert_eq!(state.focused_window, Window::OutputResults);

        state.navigate_window(Direction::Down);
        assert_eq!(state.focused_window, Window::SchemaExplorer);
    }
}
