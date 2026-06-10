use std::collections::VecDeque;

use crate::result::ResultGrid;

#[derive(Debug, Clone)]
pub struct OutputPaneState {
    pub lines: VecDeque<String>,
    pub max_lines: usize,
    pub scroll: usize,
}

impl OutputPaneState {
    pub fn new() -> Self {
        Self { lines: VecDeque::new(), max_lines: 500, scroll: 0 }
    }

    pub fn push(&mut self, message: String) {
        self.lines.push_back(message);
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        if self.lines.len() > self.max_lines / 2 {
            self.scroll = self.lines.len().saturating_sub(1);
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll + 1 < self.lines.len() {
            self.scroll += 1;
        }
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        if !self.lines.is_empty() {
            self.scroll = self.lines.len() - 1;
        }
    }
}

pub struct OutputResultsState {
    pub output: OutputPaneState,
    pub result_tabs: Vec<ResultTab>,
    pub active_tab: usize,
    pub next_result_id: usize,
}

impl OutputResultsState {
    pub fn new() -> Self {
        Self {
            output: OutputPaneState::new(),
            result_tabs: vec![ResultTab::new("Result 1".to_string())],
            active_tab: 0,
            next_result_id: 0,
        }
    }

    pub fn active_result_grid(&self) -> &ResultGrid {
        &self.result_tabs[self.active_tab_index()].grid
    }

    pub fn active_result_grid_mut(&mut self) -> &mut ResultGrid {
        let idx = self.active_tab_index();
        &mut self.result_tabs[idx].grid
    }

    fn active_tab_index(&self) -> usize {
        if self.active_tab == 0 || self.result_tabs.is_empty() {
            0
        } else {
            (self.active_tab - 1).min(self.result_tabs.len() - 1)
        }
    }

    pub fn create_result_tab(
        &mut self,
        source_split: Option<usize>,
        source_tab: Option<usize>,
    ) -> usize {
        self.next_result_id += 1;
        let title = format!("Result {}", self.next_result_id);
        let mut tab = ResultTab::new(title);
        tab.source_split = source_split;
        tab.source_tab = source_tab;
        self.result_tabs.push(tab);
        self.active_tab = self.result_tabs.len();
        self.active_tab
    }

    pub fn find_or_create_result_tab(&mut self, source_split: usize, source_tab: usize) -> usize {
        if let Some(idx) = self
            .result_tabs
            .iter()
            .position(|t| t.source_split == Some(source_split) && t.source_tab == Some(source_tab))
        {
            let tab = &mut self.result_tabs[idx];
            tab.grid = ResultGrid::new();
            self.active_tab = idx + 1;
            return self.active_tab;
        }
        if self.result_tabs.len() == 1
            && self.result_tabs[0].source_split.is_none()
            && self.result_tabs[0].source_tab.is_none()
        {
            self.next_result_id += 1;
            let title = format!("Result {}", self.next_result_id);
            let tab = &mut self.result_tabs[0];
            tab.title = title;
            tab.grid = ResultGrid::new();
            tab.source_split = Some(source_split);
            tab.source_tab = Some(source_tab);
            self.active_tab = 1;
            return 1;
        }
        self.create_result_tab(Some(source_split), Some(source_tab))
    }

    pub fn close_result_tab(&mut self, index: usize) -> bool {
        if self.result_tabs.len() <= 1 {
            return false;
        }
        self.result_tabs.remove(index);
        if self.active_tab > self.result_tabs.len() {
            self.active_tab = self.result_tabs.len();
        }
        true
    }

    pub fn next_tab(&mut self) {
        let len = 1 + self.result_tabs.len();
        self.active_tab = (self.active_tab + 1) % len;
    }

    pub fn prev_tab(&mut self) {
        let len = 1 + self.result_tabs.len();
        self.active_tab = if self.active_tab == 0 { len - 1 } else { self.active_tab - 1 };
    }
}

#[derive(Debug, Clone)]
pub struct CellPopupState {
    pub value: String,
    pub col_name: String,
    pub scroll: usize,
}

pub struct ResultTab {
    pub grid: ResultGrid,
    pub title: String,
    pub source_split: Option<usize>,
    pub source_tab: Option<usize>,
}

impl ResultTab {
    pub fn new(title: String) -> Self {
        Self { grid: ResultGrid::new(), title, source_split: None, source_tab: None }
    }
}
