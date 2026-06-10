use crate::state::{Mode, Panel};

pub struct KeyBinding {
    pub keys: &'static str,
    pub description: &'static str,
}

pub fn get_keybindings(panel: &Panel, mode: &Mode) -> Vec<KeyBinding> {
    match mode {
        Mode::Normal => match panel {
            Panel::SchemaExplorer => vec![
                KeyBinding { keys: "j / Down", description: "Next node" },
                KeyBinding { keys: "k / Up", description: "Previous node" },
                KeyBinding { keys: "l / Enter", description: "Expand table/schema" },
                KeyBinding { keys: "h", description: "Collapse node" },
                KeyBinding { keys: "R", description: "Reload schema" },
                KeyBinding { keys: "Tab", description: "Switch to Query Editor" },
                KeyBinding { keys: "K / Ctrl+Q", description: "Quick doc popup" },
                KeyBinding { keys: "?", description: "Keymap help" },
                KeyBinding { keys: ":", description: "Command palette" },
                KeyBinding { keys: "i", description: "Insert mode" },
                KeyBinding { keys: "q", description: "Quit" },
                KeyBinding { keys: "/", description: "Search (type to filter)" },
                KeyBinding { keys: "Esc", description: "Clear search" },
            ],
            Panel::QueryEditor => vec![
                KeyBinding { keys: "h/j/k/l", description: "Move cursor" },
                KeyBinding { keys: "w/b/e", description: "Word motion" },
                KeyBinding { keys: "i/I/a/A/o/O", description: "Enter Insert mode" },
                KeyBinding { keys: "Ctrl+E", description: "Execute statement" },
                KeyBinding { keys: "Ctrl+C", description: "Cancel query" },
                KeyBinding { keys: "Ctrl+P/N", description: "History prev/next" },
                KeyBinding { keys: "Ctrl+T", description: "Toggle auto-paginate" },
                KeyBinding { keys: "Ctrl+W s/v", description: "Split horizontally/vertically" },
                KeyBinding { keys: "Ctrl+W q", description: "Close editor" },
                KeyBinding { keys: "Ctrl+W h/j/k/l", description: "Focus editor" },
                KeyBinding { keys: "Tab", description: "Switch to Result Grid" },
                KeyBinding { keys: ":", description: "Command palette" },
                KeyBinding { keys: "v", description: "Visual mode" },
            ],
            Panel::ResultGrid => vec![
                KeyBinding { keys: "j/k / Down/Up", description: "Next/prev row" },
                KeyBinding { keys: "h/l / Left/Right", description: "Next/prev column" },
                KeyBinding { keys: "g g", description: "First row" },
                KeyBinding { keys: "G", description: "Last row" },
                KeyBinding { keys: "H", description: "First column" },
                KeyBinding { keys: "L", description: "Last column" },
                KeyBinding { keys: "e", description: "Edit cell" },
                KeyBinding { keys: "Enter", description: "Cell popup" },
                KeyBinding { keys: "y", description: "Copy cell" },
                KeyBinding { keys: "Y", description: "Copy row (TSV)" },
                KeyBinding { keys: "v", description: "Visual mode" },
                KeyBinding { keys: "Ctrl+D", description: "Page down" },
                KeyBinding { keys: "Ctrl+U", description: "Page up" },
                KeyBinding { keys: "Tab", description: "Switch to Output" },
                KeyBinding { keys: ":", description: "Command palette" },
            ],
            Panel::Output => vec![
                KeyBinding { keys: "j/k", description: "Scroll down/up" },
                KeyBinding { keys: "g/G", description: "Top/bottom" },
                KeyBinding { keys: "Tab", description: "Switch to Schema Explorer" },
                KeyBinding { keys: ":", description: "Command palette" },
            ],
        },
        Mode::Insert => vec![
            KeyBinding { keys: "Esc", description: "Return to Normal mode" },
            KeyBinding { keys: "Ctrl+E", description: "Execute statement" },
            KeyBinding { keys: "Ctrl+C", description: "Cancel query" },
            KeyBinding { keys: "Ctrl+P/N", description: "History prev/next" },
            KeyBinding { keys: "Backspace", description: "Delete previous char" },
            KeyBinding { keys: "Enter", description: "New line" },
            KeyBinding { keys: "Tab", description: "Insert tab (2 spaces)" },
        ],
        Mode::Command { .. } => vec![
            KeyBinding { keys: "Enter", description: "Execute command" },
            KeyBinding { keys: "Esc", description: "Cancel" },
            KeyBinding { keys: "Tab", description: "Complete command" },
            KeyBinding { keys: "Up/Down", description: "Navigate history" },
            KeyBinding { keys: "Backspace", description: "Delete previous char" },
        ],
        Mode::ConnectDialog { .. } => vec![
            KeyBinding { keys: "Tab", description: "Next field" },
            KeyBinding { keys: "Esc", description: "Cancel" },
            KeyBinding { keys: "Enter", description: "Connect" },
            KeyBinding { keys: "Left/Right", description: "Move cursor / change DB type" },
        ],
        Mode::Visual => vec![
            KeyBinding { keys: "Esc", description: "Return to Normal mode" },
            KeyBinding { keys: "h/j/k/l", description: "Extend selection" },
        ],
    }
}
