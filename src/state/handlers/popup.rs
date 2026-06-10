use crossterm::event::{KeyCode, KeyEvent};

use crate::db::client::DbCommand;
use crate::explorer::NodeKind;
use crate::state::PopupState;

impl super::super::AppState {
    pub(crate) fn handle_popup_key(&mut self, key: KeyEvent) {
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

    pub fn open_quick_doc(&mut self) {
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
}
