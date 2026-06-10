use std::cell::Cell;
use std::collections::VecDeque;
use unicode_width::UnicodeWidthStr;

const MAX_COLUMN_WIDTH: u16 = 40;
const DEFAULT_MAX_ROWS: usize = 50000;
const WIDTH_RECALC_EVERY: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub enum CellEditState {
    None,
    Editing { row: usize, col: usize, value: String, cursor: usize },
}

#[derive(Debug, Clone)]
pub struct ColumnMeta {
    pub name: String,
    #[allow(dead_code)]
    pub data_type: String,
    pub is_primary_key: bool,
}

#[derive(Debug)]
pub struct ScrollState {
    pub row_offset: usize,
    pub col_offset: usize,
    pub viewport_height: Cell<usize>,
    pub viewport_width: Cell<usize>,
}

pub struct ResultGrid {
    pub columns: Vec<ColumnMeta>,
    pub rows: VecDeque<Vec<String>>,
    pub total_rows_received: usize,
    pub scroll: ScrollState,
    pub column_widths: Vec<u16>,
    pub is_streaming: bool,
    pub selected_row: usize,
    pub selected_col: usize,
    pub max_rows: usize,
    pub last_key: Option<char>,
    pub visual_mode: bool,
    pub visual_start: Option<(usize, usize)>,
    pub null_display: String,
    #[allow(dead_code)]
    pub current_page: usize,
    pub has_more: bool,
    pub source_schema: Option<String>,
    pub source_table: Option<String>,
    pub rows_before_fetch: usize,
    pub cell_edit: CellEditState,
    rows_since_recalc: usize,
}

impl ResultGrid {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: VecDeque::new(),
            total_rows_received: 0,
            scroll: ScrollState {
                row_offset: 0,
                col_offset: 0,
                viewport_height: Cell::new(0),
                viewport_width: Cell::new(0),
            },
            column_widths: Vec::new(),
            is_streaming: false,
            selected_row: 0,
            selected_col: 0,
            max_rows: DEFAULT_MAX_ROWS,
            last_key: None,
            visual_mode: false,
            visual_start: None,
            null_display: "∅".into(),
            current_page: 0,
            has_more: false,
            source_schema: None,
            source_table: None,
            rows_before_fetch: 0,
            cell_edit: CellEditState::None,
            rows_since_recalc: 0,
        }
    }

    pub fn set_columns(&mut self, columns: Vec<ColumnMeta>) {
        self.column_widths =
            columns.iter().map(|c| (c.name.len() as u16).min(MAX_COLUMN_WIDTH)).collect();
        self.columns = columns;
        self.rows_since_recalc = 0;
    }

    pub fn add_row(&mut self, cells: Vec<String>) {
        self.total_rows_received += 1;
        while self.rows.len() >= self.max_rows {
            self.rows.pop_front();
            if self.selected_row > 0 {
                self.selected_row -= 1;
            }
            if self.scroll.row_offset > 0 {
                self.scroll.row_offset -= 1;
            }
        }
        self.rows.push_back(cells);
        self.rows_since_recalc += 1;
        if self.rows_since_recalc.is_multiple_of(WIDTH_RECALC_EVERY) {
            self.recalculate_widths();
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.columns.clear();
        self.rows.clear();
        self.total_rows_received = 0;
        self.column_widths.clear();
        self.scroll.row_offset = 0;
        self.scroll.col_offset = 0;
        self.selected_row = 0;
        self.selected_col = 0;
        self.visual_mode = false;
        self.visual_start = None;
        self.current_page = 0;
        self.has_more = false;
        self.source_schema = None;
        self.source_table = None;
        self.cell_edit = CellEditState::None;
        self.rows_since_recalc = 0;
    }

    pub fn recalculate_widths(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        let sample_count = (200usize).min(self.rows.len());
        self.column_widths = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let header_w = col.name.len();
                let content_w = self
                    .rows
                    .iter()
                    .take(sample_count)
                    .map(|row| {
                        row.get(i)
                            .map(|cell| {
                                if cell.is_empty() {
                                    self.null_display.width()
                                } else {
                                    cell.width()
                                }
                            })
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);
                (header_w.max(content_w) as u16).min(MAX_COLUMN_WIDTH)
            })
            .collect();
    }

    pub fn move_selection(&mut self, down: i32, right: i32) {
        if self.rows.is_empty() {
            return;
        }

        if down != 0 {
            let new_row = if down > 0 {
                (self.selected_row + down as usize).min(self.rows.len().saturating_sub(1))
            } else {
                self.selected_row.saturating_sub((-down) as usize)
            };
            self.selected_row = new_row;
        }

        if right != 0 && !self.columns.is_empty() {
            let new_col = if right > 0 {
                (self.selected_col + right as usize).min(self.columns.len().saturating_sub(1))
            } else {
                self.selected_col.saturating_sub((-right) as usize)
            };
            self.selected_col = new_col;
        }

        self.scroll_to_selection();
    }

    pub fn needs_next_page(&self) -> bool {
        if !self.has_more || self.rows.is_empty() {
            return false;
        }
        let last_visible =
            self.scroll.row_offset + self.scroll.viewport_height.get().saturating_sub(1);
        self.selected_row >= last_visible.min(self.rows.len().saturating_sub(1))
    }

    pub fn page_down(&mut self) {
        let rows = self.scroll.viewport_height.get().max(1);
        self.selected_row = (self.selected_row + rows).min(self.rows.len().saturating_sub(1));
        self.scroll.row_offset =
            (self.scroll.row_offset + rows).min(self.rows.len().saturating_sub(1));
        self.scroll_to_selection();
    }

    pub fn page_up(&mut self) {
        let rows = self.scroll.viewport_height.get().max(1);
        self.selected_row = self.selected_row.saturating_sub(rows);
        self.scroll.row_offset = self.scroll.row_offset.saturating_sub(rows);
        self.scroll_to_selection();
    }

    pub fn first_row(&mut self) {
        self.selected_row = 0;
        self.scroll.row_offset = 0;
    }

    pub fn last_row(&mut self) {
        if !self.rows.is_empty() {
            let last = self.rows.len() - 1;
            self.selected_row = last;
            self.scroll.row_offset =
                last.saturating_sub(self.scroll.viewport_height.get().max(1) - 1);
        }
    }

    pub fn first_col(&mut self) {
        self.selected_col = 0;
        self.scroll.col_offset = 0;
    }

    pub fn last_col(&mut self) {
        if !self.columns.is_empty() {
            self.selected_col = self.columns.len() - 1;
        }
    }

    fn scroll_to_selection(&mut self) {
        let vh = self.scroll.viewport_height.get().max(1);

        if self.selected_row < self.scroll.row_offset {
            self.scroll.row_offset = self.selected_row;
        }
        if self.selected_row >= self.scroll.row_offset + vh {
            self.scroll.row_offset = self.selected_row.saturating_sub(vh - 1);
        }

        if self.selected_col < self.scroll.col_offset {
            self.scroll.col_offset = self.selected_col;
        }
        self.ensure_col_visible();
    }

    fn ensure_col_visible(&mut self) {
        // Simple: keep selected_col within col_offset .. col_offset+n
        // The actual visible column calculation happens in render
        loop {
            if self.selected_col < self.scroll.col_offset {
                if self.scroll.col_offset > 0 {
                    self.scroll.col_offset -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn selected_cell_value(&self) -> Option<&str> {
        self.rows
            .get(self.selected_row)
            .and_then(|row| row.get(self.selected_col))
            .map(|s| s.as_str())
    }

    pub fn selected_row_tsv(&self) -> String {
        match self.rows.get(self.selected_row) {
            Some(row) => row.join("\t"),
            None => String::new(),
        }
    }

    pub fn visual_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let start = self.visual_start?;
        let end = (self.selected_row, self.selected_col);
        let r1 = start.0.min(end.0);
        let r2 = start.0.max(end.0);
        let c1 = start.1.min(end.1);
        let c2 = start.1.max(end.1);
        Some(((r1, c1), (r2, c2)))
    }

    pub fn visual_range_tsv(&self) -> Option<String> {
        let ((r1, c1), (r2, c2)) = self.visual_range()?;
        let mut lines = Vec::new();
        for row in r1..=r2 {
            if let Some(cells) = self.rows.get(row) {
                let slice: Vec<&str> = cells[c1..=c2.min(cells.len().saturating_sub(1))]
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                lines.push(slice.join("\t"));
            }
        }
        if lines.is_empty() { None } else { Some(lines.join("\n")) }
    }

    #[allow(dead_code)]
    pub fn visible_rows(&self) -> impl Iterator<Item = (usize, &Vec<String>)> {
        let start = self.scroll.row_offset;
        let end = (start + self.scroll.viewport_height.get().max(1)).min(self.rows.len());
        self.rows.iter().skip(start).take(end - start).enumerate()
    }

    pub fn visible_col_range(&self) -> (usize, usize) {
        if self.columns.is_empty() {
            return (0, 0);
        }
        let mut used: u16 = 0;
        let mut end = self.scroll.col_offset;
        let max_width = self.scroll.viewport_width.get().max(1) as u16;

        for (i, w) in self.column_widths.iter().enumerate().skip(self.scroll.col_offset) {
            if used + w > max_width && end > self.scroll.col_offset {
                break;
            }
            used += w;
            end = i + 1;
        }
        (self.scroll.col_offset, end)
    }

    pub fn enter_edit(&mut self, row: usize, col: usize, current_value: &str) {
        self.cell_edit = CellEditState::Editing {
            row,
            col,
            value: current_value.to_string(),
            cursor: current_value.len(),
        };
    }

    pub fn edit_insert_char(&mut self, c: char) {
        if let CellEditState::Editing { ref mut value, ref mut cursor, .. } = self.cell_edit {
            value.insert(*cursor, c);
            *cursor += 1;
        }
    }

    pub fn edit_delete_backward(&mut self) {
        if let CellEditState::Editing { ref mut value, ref mut cursor, .. } = self.cell_edit
            && *cursor > 0
        {
            *cursor -= 1;
            value.remove(*cursor);
        }
    }

    pub fn edit_move_cursor(&mut self, right: bool) {
        if let CellEditState::Editing { ref mut cursor, ref value, .. } = self.cell_edit {
            if right {
                if *cursor < value.len() {
                    *cursor += 1;
                }
            } else if *cursor > 0 {
                *cursor -= 1;
            }
        }
    }

    pub fn commit_edit(&self) -> Option<(&str, &str)> {
        match &self.cell_edit {
            CellEditState::Editing { row, col, value, .. } => {
                let old_value = self.rows.get(*row)?.get(*col)?;
                Some((old_value.as_str(), value.as_str()))
            },
            CellEditState::None => None,
        }
    }

    pub fn cancel_edit(&mut self) {
        self.cell_edit = CellEditState::None;
    }

    pub fn is_editing(&self) -> bool {
        !matches!(self.cell_edit, CellEditState::None)
    }

    pub fn pk_where_clause(
        &self,
        row_idx: usize,
        columns: &[ColumnMeta],
    ) -> Option<Vec<(String, String)>> {
        let row = self.rows.get(row_idx)?;
        let mut clauses = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            if col.is_primary_key {
                let col_name = col.name.clone();
                let value = row.get(i).cloned().unwrap_or_default();
                clauses.push((col_name, value));
            }
        }
        if clauses.is_empty() { None } else { Some(clauses) }
    }
}
