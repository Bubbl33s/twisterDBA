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
