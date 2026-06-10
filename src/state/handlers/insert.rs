use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::key_matches_config;

impl super::super::AppState {
    pub(crate) fn handle_insert_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = super::super::Mode::Normal;
            return;
        }
        if self.focused_window == crate::state::Window::QueryEditor {
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
}
