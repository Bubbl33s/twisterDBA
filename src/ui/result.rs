use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, TableState},
};

use crate::state::AppState;
use crate::theme::Theme;

pub fn render_result_grid(f: &mut Frame, area: Rect, state: &AppState, title: &str, theme: &Theme) {
    let block = Block::default().title(title).style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height.saturating_sub(2) as usize;
    let viewport_width = inner.width as usize;

    let grid = state.active_result_grid();
    grid.scroll.viewport_height.set(viewport_height);
    grid.scroll.viewport_width.set(viewport_width);

    if grid.rows.is_empty() {
        if state.focused_editor().executing {
            let spinner = state.spinner_char();
            f.render_widget(
                Paragraph::new(Text::from(format!("{} Running query...", spinner))).block(block),
                area,
            );
        } else if let Some(ref err) = state.last_query_error {
            f.render_widget(
                Paragraph::new(Text::from(format!("Error: {}", err)))
                    .style(Style::default().fg(Color::Red))
                    .block(block),
                area,
            );
        } else {
            f.render_widget(Paragraph::new(Text::from("(no results)")).block(block), area);
        }
        return;
    }

    let viewport_height = inner.height.saturating_sub(2) as usize;
    let viewport_width = inner.width as usize;

    if viewport_height == 0 || viewport_width == 0 {
        return;
    }

    let (col_start, col_end) = grid.visible_col_range();
    let vis_cols = &grid.columns[col_start..col_end];
    let vis_widths: Vec<Constraint> =
        grid.column_widths[col_start..col_end].iter().map(|w| Constraint::Length(*w)).collect();

    let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let header_cells: Vec<Cell> =
        vis_cols.iter().map(|c| Cell::from(c.name.as_str()).style(header_style)).collect();
    let header = Row::new(header_cells).height(1);

    let row_start = grid.scroll.row_offset;
    let row_end = (row_start + viewport_height).min(grid.rows.len());
    let visible_rows: Vec<Row> = grid
        .rows
        .iter()
        .skip(row_start)
        .take(row_end - row_start)
        .enumerate()
        .map(|(rel_idx, cells)| {
            let abs_idx = row_start + rel_idx;
            let row_style = if abs_idx == grid.selected_row {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let rendered_cells: Vec<Cell> = cells[col_start..col_end]
                .iter()
                .enumerate()
                .map(|(col_rel, value)| {
                    let abs_col = col_start + col_rel;
                    let is_selected = abs_idx == grid.selected_row && abs_col == grid.selected_col;

                    let display =
                        if value.is_empty() { grid.null_display.clone() } else { value.clone() };

                    if is_selected {
                        Cell::from(display)
                            .style(row_style.bg(Color::DarkGray).add_modifier(Modifier::REVERSED))
                    } else {
                        Cell::from(display).style(row_style)
                    }
                })
                .collect();

            Row::new(rendered_cells)
        })
        .collect();

    let mut table_state = TableState::default();
    let selected_rel = grid.selected_row.saturating_sub(row_start);
    table_state.select(Some(selected_rel));

    let table = Table::new(visible_rows, vis_widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(table, area, &mut table_state);
}
