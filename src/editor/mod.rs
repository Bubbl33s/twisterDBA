use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::cell::RefCell;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use unicode_segmentation::UnicodeSegmentation;

use crate::db::client::DbCommand;
use crate::editor::tree::TsParser;
use crate::state::OutputPaneState;
use crate::theme::Theme;

pub mod tree;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

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

const MAX_HISTORY: usize = 1000;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub sql: String,
}

pub struct QueryHistory {
    entries: VecDeque<HistoryEntry>,
    position: Option<usize>,
    draft: Option<String>,
}

impl QueryHistory {
    pub fn new() -> Self {
        Self { entries: VecDeque::new(), position: None, draft: None }
    }

    pub fn push(&mut self, sql: String) {
        if self.entries.back().is_none_or(|e| e.sql != sql) {
            self.entries.push_back(HistoryEntry { sql });
            if self.entries.len() > MAX_HISTORY {
                self.entries.pop_front();
            }
        }
        self.position = None;
        self.draft = None;
    }

    pub fn navigate_previous(&mut self, current: &str) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }
        match self.position {
            None => {
                self.draft = Some(current.to_string());
                let idx = self.entries.len() - 1;
                self.position = Some(idx);
                Some(self.entries[idx].sql.clone())
            },
            Some(pos) if pos > 0 => {
                let idx = pos - 1;
                self.position = Some(idx);
                Some(self.entries[idx].sql.clone())
            },
            _ => None,
        }
    }

    pub fn navigate_next(&mut self) -> Option<String> {
        match self.position {
            None => None,
            Some(pos) if pos + 1 < self.entries.len() => {
                let idx = pos + 1;
                self.position = Some(idx);
                Some(self.entries[idx].sql.clone())
            },
            Some(_) => {
                let draft = self.draft.take();
                self.position = None;
                draft
            },
        }
    }
}

pub struct SqlEditor {
    pub buffer: TextBuffer,
    pub history: QueryHistory,
    pub executing: bool,
    current_cancel: Option<CancellationToken>,
    ts_parser: RefCell<TsParser>,
    #[allow(dead_code)]
    pub theme: Theme,
    pub auto_paginate: bool,
    pub page_size: usize,
    pub last_executed_sql: Option<String>,
    pub current_page: usize,
    pub output_pane: OutputPaneState,
}

impl SqlEditor {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            history: QueryHistory::new(),
            executing: false,
            current_cancel: None,
            ts_parser: RefCell::new(TsParser::new()),
            theme: Theme::darcula(),
            auto_paginate: true,
            page_size: 200,
            last_executed_sql: None,
            current_page: 0,
            output_pane: OutputPaneState::new(),
        }
    }

    pub fn execute(&mut self, db_tx: &mpsc::UnboundedSender<DbCommand>) -> bool {
        if self.executing {
            return false;
        }
        let sql = self.extract_active_statement().unwrap_or_else(|| self.buffer.get_content());
        if sql.trim().is_empty() {
            return false;
        }

        self.cancel_query();
        let cancel = CancellationToken::new();
        self.current_cancel = Some(cancel.clone());

        self.history.push(sql.clone());

        let (final_sql, auto_paginate_applied) =
            if self.auto_paginate && Self::is_select_query(&sql) && !Self::has_limit_clause(&sql) {
                self.last_executed_sql = Some(sql.clone());
                self.current_page = 0;
                (Self::inject_pagination(&sql, 0, self.page_size), true)
            } else {
                self.last_executed_sql = None;
                self.current_page = 0;
                (sql.clone(), false)
            };

        let _ = db_tx.send(DbCommand::ExecuteQuery {
            sql: final_sql,
            cancel,
            auto_paginate: auto_paginate_applied,
            page_size: self.page_size,
        });
        self.executing = true;
        true
    }

    pub fn cancel_query(&mut self) {
        if let Some(token) = self.current_cancel.take() {
            token.cancel();
        }
        self.executing = false;
        self.last_executed_sql = None;
    }

    pub fn mark_completed(&mut self) {
        self.current_cancel = None;
        self.executing = false;
    }

    pub fn inject_pagination(sql: &str, page: usize, page_size: usize) -> String {
        let offset = page * page_size;
        let limit = page_size + 1;
        let clean_sql = sql.trim().trim_end_matches(';').trim();
        format!("SELECT * FROM ({}) AS _twister_page LIMIT {} OFFSET {}", clean_sql, limit, offset)
    }

    fn is_select_query(sql: &str) -> bool {
        let trimmed = sql.trim_start();
        let lower = trimmed.to_lowercase();
        lower.starts_with("select") || lower.starts_with("with")
    }

    fn has_limit_clause(sql: &str) -> bool {
        let upper = sql.to_uppercase();
        let bytes = upper.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'L'
                && i + 4 < bytes.len()
                && bytes[i + 1] == b'I'
                && bytes[i + 2] == b'M'
                && bytes[i + 3] == b'I'
                && bytes[i + 4] == b'T'
            {
                let is_start =
                    i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
                let is_end = i + 5 >= bytes.len()
                    || !bytes[i + 5].is_ascii_alphanumeric() && bytes[i + 5] != b'_';
                if is_start && is_end {
                    return true;
                }
            }
            i += 1;
        }
        false
    }

    pub fn fetch_next_page(
        &mut self,
        db_tx: &mpsc::UnboundedSender<DbCommand>,
        page: usize,
    ) -> bool {
        if self.executing {
            return false;
        }
        if let Some(ref sql) = self.last_executed_sql.clone() {
            self.current_page = page;
            let paginated = Self::inject_pagination(sql, page, self.page_size);
            let cancel = CancellationToken::new();
            self.current_cancel = Some(cancel.clone());
            let _ = db_tx.send(DbCommand::FetchNextPage { page, sql: paginated, cancel });
            self.executing = true;
            true
        } else {
            false
        }
    }

    pub fn highlight_lines(&self) -> Vec<Vec<Span<'static>>> {
        let source = self.buffer.get_content();
        let tree = {
            let mut parser = self.ts_parser.borrow_mut();
            parser.parse(&source)
        };
        let tree = match tree {
            Some(t) => t,
            None => {
                return source.lines().map(|l| vec![Span::raw(l.to_string())]).collect();
            },
        };
        let parser = self.ts_parser.borrow();
        let mut result: Vec<Vec<Span<'static>>> = Vec::new();
        for (i, line) in source.lines().enumerate() {
            result.push(parser.highlight_line(&tree, &source, line, i));
        }
        result
    }

    pub fn extract_active_statement(&self) -> Option<String> {
        let source = self.buffer.get_content();
        let cursor_byte = self.buffer.cursor_byte_offset();
        let mut ts_parser = TsParser::new();
        let range = ts_parser.find_statement_at(&source, cursor_byte)?;
        let extracted = ts_parser.extract_text(&source, range).trim().to_string();
        if extracted.is_empty() { None } else { Some(extracted) }
    }

    pub fn handle_normal_key(&mut self, key: &KeyEvent) -> bool {
        if self.executing {
            return false;
        }
        self.buffer.handle_normal_key(key)
    }

    pub fn handle_insert_key(&mut self, key: &KeyEvent) -> bool {
        if self.executing {
            return false;
        }
        self.buffer.handle_insert_key(key)
    }
}

pub fn render_line_with_cursor(
    line: &str,
    cursor_col: usize,
    spans: Vec<Span<'static>>,
) -> Line<'static> {
    if (line.is_empty() && cursor_col == 0) || spans.is_empty() {
        return if cursor_col == 0 && line.is_empty() {
            Line::from(vec![Span::styled(
                " ",
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
            )])
        } else {
            Line::from(spans)
        };
    }

    let mut char_count: usize = 0;
    let mut new_spans: Vec<Span<'static>> = Vec::new();
    let mut found = false;

    for span in spans {
        if found {
            new_spans.push(span);
            continue;
        }

        let span_chars: Vec<&str> = span.content.graphemes(true).collect();
        let span_len = span_chars.len();

        if char_count + span_len > cursor_col {
            let cursor_offset = cursor_col - char_count;

            let before: String = span_chars[..cursor_offset].join("");
            let cursor_char = span_chars[cursor_offset].to_string();
            let after: String = span_chars[cursor_offset + 1..].join("");

            if !before.is_empty() {
                new_spans.push(Span::styled(before, span.style));
            }
            new_spans.push(Span::styled(
                cursor_char,
                span.style.bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
            ));
            if !after.is_empty() {
                new_spans.push(Span::styled(after, span.style));
            }

            found = true;
        } else {
            new_spans.push(span);
            char_count += span_len;
        }
    }

    if !found && cursor_col >= char_count {
        new_spans.push(Span::styled(
            " ",
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
        ));
    }

    Line::from(new_spans)
}
