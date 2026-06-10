use crossterm::event::{KeyCode, KeyEvent};

impl super::super::AppState {
    pub(crate) fn handle_visual_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = super::super::Mode::Normal;
        }
    }
}
