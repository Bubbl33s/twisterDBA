pub(crate) mod command;
mod connect;
mod explorer;
mod insert;
mod normal;
mod popup;
mod result;
mod visual;

use crossterm::event::KeyEvent;

use crate::state::Mode;

impl super::AppState {
    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Insert => self.handle_insert_key(key),
            Mode::Command { .. } => self.handle_command_key(key),
            Mode::ConnectDialog { .. } => self.handle_connect_dialog_key(key),
            Mode::Visual => self.handle_visual_key(key),
        }
    }
}

pub(crate) fn key_event_to_string(key: &KeyEvent) -> String {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Char(c) => c.to_lowercase().to_string(),
        _ => String::new(),
    }
}

pub(crate) fn key_matches_config(key: &KeyEvent, binding: &str) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};
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
