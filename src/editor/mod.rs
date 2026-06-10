pub mod buffer;
pub mod history;
pub mod render;
pub mod tree;

use std::cell::RefCell;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::db::client::DbCommand;
use crate::editor::tree::TsParser;
use crate::theme::Theme;

pub use buffer::TextBuffer;
pub use history::QueryHistory;
pub use render::render_line_with_cursor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
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

    pub fn highlight_lines(&self) -> Vec<Vec<ratatui::text::Span<'static>>> {
        let source = self.buffer.get_content();
        let tree = {
            let mut parser = self.ts_parser.borrow_mut();
            parser.parse(&source)
        };
        let tree = match tree {
            Some(t) => t,
            None => {
                return source
                    .lines()
                    .map(|l| vec![ratatui::text::Span::raw(l.to_string())])
                    .collect();
            },
        };
        let parser = self.ts_parser.borrow();
        let mut result: Vec<Vec<ratatui::text::Span<'static>>> = Vec::new();
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
