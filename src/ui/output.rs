use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::state::{AppState, OutputPaneState};
use crate::theme::Theme;

pub fn render_output_panel(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let tab_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let show_output = state.output_results.active_tab == 0;

    render_output_tab_bar(f, tab_chunks[0], state, theme);

    if show_output {
        render_output_pane(f, tab_chunks[1], &state.output_results.output, theme);
    } else {
        let grid = state.active_result_grid();
        let grid_title = if grid.is_streaming {
            format!(" Results ({} / streaming...) ", grid.total_rows_received)
        } else {
            format!(" Results ({} rows) ", grid.total_rows_received)
        };
        super::result::render_result_grid(f, tab_chunks[1], state, &grid_title, theme);
    }
}

pub fn render_output_tab_bar(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let show_output = state.output_results.active_tab == 0;
    let output_style = if show_output {
        Style::default().fg(Color::Black).bg(Color::White)
    } else {
        Style::default().fg(Color::Gray).bg(theme.editor_bg)
    };

    let mut spans = vec![Span::styled(" Output ", output_style)];

    for (i, result_tab) in state.output_results.result_tabs.iter().enumerate() {
        let result_tab_idx = i + 1;
        let is_active = !show_output && result_tab_idx == state.output_results.active_tab;
        let tab_style = if is_active {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::Gray).bg(theme.editor_bg)
        };
        spans.push(Span::styled(" ", Style::default().bg(theme.editor_bg)));
        spans.push(Span::styled(format!(" {} ", result_tab.title), tab_style));
    }

    let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
    spans.push(Span::styled(
        " ".repeat(area.width.saturating_sub(used_width as u16) as usize),
        Style::default().bg(theme.editor_bg),
    ));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn render_output_pane(f: &mut Frame, area: Rect, output: &OutputPaneState, theme: &Theme) {
    let block = Block::default().title(" Output ").style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height.saturating_sub(1) as usize;

    if output.lines.is_empty() {
        f.render_widget(Paragraph::new(Text::from("(no output)")).block(block), area);
        return;
    }

    let scroll_top = output.scroll.saturating_sub(viewport_height.saturating_sub(1));
    let end = (scroll_top + viewport_height).min(output.lines.len());

    let lines: Vec<Line> = output
        .lines
        .iter()
        .skip(scroll_top)
        .take(end.saturating_sub(scroll_top))
        .map(|s| {
            if s.contains("ERROR") {
                Line::from(Span::styled(s.clone(), Style::default().fg(Color::Red)))
            } else {
                Line::from(Span::raw(s.clone()))
            }
        })
        .collect();

    f.render_widget(Paragraph::new(lines).block(block), area);
}
