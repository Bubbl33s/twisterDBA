use std::collections::VecDeque;

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
