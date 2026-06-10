#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command { buffer: String },
    ConnectDialog { form: super::connection::ConnectForm },
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    SchemaExplorer,
    QueryEditor,
    OutputResults,
}
