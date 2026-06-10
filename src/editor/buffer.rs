use crossterm::event::{KeyCode, KeyEvent};
use unicode_segmentation::UnicodeSegmentation;

use super::Direction;

pub struct TextBuffer {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    yank: Option<String>,
    tab_width: usize,
    last_key: Option<char>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            yank: None,
            tab_width: 4,
            last_key: None,
        }
    }

    fn grapheme_count(line: &str) -> usize {
        line.graphemes(true).count()
    }

    fn byte_offset(line: &str, grapheme_idx: usize) -> usize {
        line.grapheme_indices(true).nth(grapheme_idx).map(|(i, _)| i).unwrap_or(line.len())
    }

    fn line_len(&self, row: usize) -> usize {
        Self::grapheme_count(&self.lines[row])
    }

    pub fn set_content(&mut self, sql: &str) {
        if sql.is_empty() {
            self.lines = vec![String::new()];
            self.cursor_row = 0;
            self.cursor_col = 0;
            return;
        }
        self.lines = sql.split('\n').map(|s| s.to_string()).collect();
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.line_len(self.cursor_row);
        self.scroll_offset = 0;
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn cursor_byte_offset(&self) -> usize {
        let mut offset: usize = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if i == self.cursor_row {
                offset += Self::byte_offset(line, self.cursor_col);
                break;
            }
            offset += line.len() + 1;
        }
        offset
    }

    fn clamp_col(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col > len {
            self.cursor_col = len;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let offset = Self::byte_offset(&self.lines[self.cursor_row], self.cursor_col);
        self.lines[self.cursor_row].insert(offset, c);
        self.cursor_col += 1;
    }

    pub fn delete_backward(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                let current = self.lines.remove(self.cursor_row);
                self.cursor_row -= 1;
                self.cursor_col = self.line_len(self.cursor_row);
                self.lines[self.cursor_row].push_str(&current);
            }
        } else {
            self.cursor_col -= 1;
            let offset = Self::byte_offset(&self.lines[self.cursor_row], self.cursor_col);
            let grapheme =
                self.lines[self.cursor_row].graphemes(true).nth(self.cursor_col).unwrap_or("");
            let end = offset + grapheme.len();
            self.lines[self.cursor_row].replace_range(offset..end, "");
        }
    }

    pub fn delete_forward(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col >= len {
            if self.cursor_row + 1 < self.lines.len() {
                let next = self.lines.remove(self.cursor_row + 1);
                self.lines[self.cursor_row].push_str(&next);
            }
        } else {
            let offset = Self::byte_offset(&self.lines[self.cursor_row], self.cursor_col);
            let grapheme =
                self.lines[self.cursor_row].graphemes(true).nth(self.cursor_col).unwrap_or("");
            let end = offset + grapheme.len();
            self.lines[self.cursor_row].replace_range(offset..end, "");
        }
    }

    pub fn insert_newline(&mut self) {
        let offset = Self::byte_offset(&self.lines[self.cursor_row], self.cursor_col);
        let rest = self.lines[self.cursor_row].split_off(offset);
        self.lines.insert(self.cursor_row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn insert_newline_below(&mut self) {
        let end = self.line_len(self.cursor_row);
        self.cursor_col = end;
        self.insert_newline();
    }

    pub fn insert_newline_above(&mut self) {
        self.cursor_col = 0;
        self.insert_newline();
        if self.cursor_row > 0 {
            let newline = self.lines.remove(self.cursor_row);
            self.lines.insert(self.cursor_row, newline);
            self.cursor_row -= 1;
            self.cursor_col = 0;
        }
    }

    fn ensure_scroll(&mut self) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        match dir {
            Direction::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.clamp_col();
                    if self.cursor_row < self.scroll_offset {
                        self.scroll_offset = self.cursor_row;
                    }
                }
            },
            Direction::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.clamp_col();
                    self.ensure_scroll();
                }
            },
            Direction::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.line_len(self.cursor_row);
                    if self.cursor_row < self.scroll_offset {
                        self.scroll_offset = self.cursor_row;
                    }
                }
            },
            Direction::Right => {
                let len = self.line_len(self.cursor_row);
                if self.cursor_col < len {
                    self.cursor_col += 1;
                } else if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                    self.ensure_scroll();
                }
            },
        }
    }

    pub fn line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn line_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
    }

    pub fn buffer_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn buffer_end(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.line_len(self.cursor_row);
    }

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    pub fn word_forward(&mut self) {
        let line = &self.lines[self.cursor_row];
        let len = Self::grapheme_count(line);
        if self.cursor_col >= len {
            if self.cursor_row + 1 < self.lines.len() {
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.ensure_scroll();
                self.word_forward();
            }
            return;
        }

        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let col = self.cursor_col;

        if col < len {
            let first = graphemes[col].chars().next().unwrap_or(' ');
            if Self::is_word_char(first) {
                self.cursor_col += 1;
                while self.cursor_col < len {
                    let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
                    if !Self::is_word_char(c) {
                        break;
                    }
                    self.cursor_col += 1;
                }
            }
            while self.cursor_col < len {
                let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
                if Self::is_word_char(c) {
                    break;
                }
                self.cursor_col += 1;
            }
        }
    }

    pub fn word_back(&mut self) {
        let line = &self.lines[self.cursor_row];
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = Self::grapheme_count(&self.lines[self.cursor_row]);
                if self.cursor_row < self.scroll_offset {
                    self.scroll_offset = self.cursor_row;
                }
                self.word_back();
            }
            return;
        }

        let graphemes: Vec<&str> = line.graphemes(true).collect();
        self.cursor_col -= 1;

        while self.cursor_col > 0 {
            let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
            let prev = graphemes[self.cursor_col - 1].chars().next().unwrap_or(' ');
            if Self::is_word_char(prev) && !Self::is_word_char(c)
                || !Self::is_word_char(prev) && !Self::is_word_char(c) && prev != c
            {
                break;
            }
            self.cursor_col -= 1;
        }

        if self.cursor_col > 0 {
            let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
            if !Self::is_word_char(c) {
                while self.cursor_col < Self::grapheme_count(line) {
                    let d = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
                    if Self::is_word_char(d) {
                        break;
                    }
                    self.cursor_col += 1;
                }
            }
        }
    }

    pub fn word_end(&mut self) {
        let line = &self.lines[self.cursor_row];
        let len = Self::grapheme_count(line);
        if self.cursor_col >= len {
            if self.cursor_row + 1 < self.lines.len() {
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.ensure_scroll();
                self.word_end();
            }
            return;
        }

        let graphemes: Vec<&str> = line.graphemes(true).collect();

        let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
        if Self::is_word_char(c) {
            while self.cursor_col + 1 < len {
                let next = graphemes[self.cursor_col + 1].chars().next().unwrap_or(' ');
                if !Self::is_word_char(next) {
                    break;
                }
                self.cursor_col += 1;
            }
            return;
        }

        while self.cursor_col < len {
            let c = graphemes[self.cursor_col].chars().next().unwrap_or(' ');
            if Self::is_word_char(c) {
                break;
            }
            self.cursor_col += 1;
        }
        while self.cursor_col + 1 < len {
            let next = graphemes[self.cursor_col + 1].chars().next().unwrap_or(' ');
            if !Self::is_word_char(next) {
                break;
            }
            self.cursor_col += 1;
        }
    }

    pub fn delete_line(&mut self) {
        self.yank = Some(self.lines[self.cursor_row].clone());
        if self.lines.len() == 1 {
            self.lines[0] = String::new();
            self.cursor_col = 0;
        } else {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            self.cursor_col = 0;
        }
    }

    pub fn yank_line(&mut self) {
        self.yank = Some(self.lines[self.cursor_row].clone());
    }

    pub fn paste(&mut self) {
        if let Some(ref text) = self.yank.clone() {
            if self.cursor_row < self.lines.len() {
                self.lines.insert(self.cursor_row + 1, text.clone());
            }
            self.cursor_row = (self.cursor_row + 1).min(self.lines.len() - 1);
            self.cursor_col = 0;
        }
    }

    pub fn handle_normal_key(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('g') => {
                if self.last_key == Some('g') {
                    self.buffer_start();
                    self.last_key = None;
                } else {
                    self.last_key = Some('g');
                }
                true
            },
            KeyCode::Char('d') => {
                if self.last_key == Some('d') {
                    self.delete_line();
                    self.last_key = None;
                } else {
                    self.last_key = Some('d');
                }
                true
            },
            KeyCode::Char('y') => {
                if self.last_key == Some('y') {
                    self.yank_line();
                    self.last_key = None;
                } else {
                    self.last_key = Some('y');
                }
                true
            },
            other => {
                self.last_key = None;
                match other {
                    KeyCode::Char('h') => {
                        self.move_cursor(Direction::Left);
                        true
                    },
                    KeyCode::Char('j') => {
                        self.move_cursor(Direction::Down);
                        true
                    },
                    KeyCode::Char('k') => {
                        self.move_cursor(Direction::Up);
                        true
                    },
                    KeyCode::Char('l') => {
                        self.move_cursor(Direction::Right);
                        true
                    },
                    KeyCode::Char('w') => {
                        self.word_forward();
                        true
                    },
                    KeyCode::Char('b') => {
                        self.word_back();
                        true
                    },
                    KeyCode::Char('e') => {
                        self.word_end();
                        true
                    },
                    KeyCode::Char('0') => {
                        self.line_start();
                        true
                    },
                    KeyCode::Char('$') => {
                        self.line_end();
                        true
                    },
                    KeyCode::Char('G') => {
                        self.buffer_end();
                        true
                    },
                    KeyCode::Char('p') => {
                        self.paste();
                        true
                    },
                    _ => false,
                }
            },
        }
    }

    pub fn handle_insert_key(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            },
            KeyCode::Backspace => {
                self.delete_backward();
                true
            },
            KeyCode::Delete => {
                self.delete_forward();
                true
            },
            KeyCode::Enter => {
                self.insert_newline();
                true
            },
            KeyCode::Tab => {
                for _ in 0..self.tab_width {
                    self.insert_char(' ');
                }
                true
            },
            KeyCode::Up => {
                self.move_cursor(Direction::Up);
                true
            },
            KeyCode::Down => {
                self.move_cursor(Direction::Down);
                true
            },
            KeyCode::Left => {
                self.move_cursor(Direction::Left);
                true
            },
            KeyCode::Right => {
                self.move_cursor(Direction::Right);
                true
            },
            KeyCode::Home => {
                self.line_start();
                true
            },
            KeyCode::End => {
                self.line_end();
                true
            },
            _ => false,
        }
    }
}
