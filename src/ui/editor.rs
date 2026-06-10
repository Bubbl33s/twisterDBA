use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::editor;
use crate::state::{AppState, Mode, Window};
use crate::theme::Theme;

pub fn render_single_editor(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    editor_idx: usize,
    theme: &Theme,
) {
    let editor = &state.editor_splits[editor_idx].active_editor();
    let is_focused = editor_idx == state.active_split;

    let show_tab_bar = state.editor_splits[editor_idx].tabs.len() > 1;
    let chunks = if show_tab_bar {
        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area)
    } else {
        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(0),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area)
    };
    if show_tab_bar {
        render_editor_tab_bar(f, chunks[0], state, editor_idx, theme);
    }

    let editor_title = if editor.executing {
        format!(" SQL Editor {} [running...] ", editor_idx + 1)
    } else {
        format!(" SQL Editor {} ", editor_idx + 1)
    };

    render_sql_editor(f, chunks[1], state, editor_idx, is_focused, &editor_title, theme);
}

pub fn render_editor_tab_bar(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    editor_idx: usize,
    theme: &Theme,
) {
    let split = &state.editor_splits[editor_idx];
    let is_split_focused =
        state.focused_window == Window::QueryEditor && state.active_split == editor_idx;
    let mut spans: Vec<Span> = Vec::new();
    for (i, tab) in split.tabs.iter().enumerate() {
        let is_active = is_split_focused && i == split.active_tab;
        let label = editor_tab_label(tab, i);
        let style = if is_active {
            Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(theme.editor_bg)
        };
        if i > 0 {
            spans.push(Span::styled(" ", Style::default().bg(theme.editor_bg)));
        }
        spans.push(Span::styled(format!(" {} ", label), style));
    }
    let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
    spans.push(Span::styled(
        " ".repeat(area.width.saturating_sub(used_width as u16) as usize),
        Style::default().bg(theme.editor_bg),
    ));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn editor_tab_label(tab: &crate::editor::SqlEditor, idx: usize) -> String {
    let first_line =
        tab.buffer.get_content().lines().next().map(str::trim).unwrap_or("").to_string();
    if first_line.is_empty() {
        format!("Query {}", idx + 1)
    } else {
        let max_chars = 24;
        let truncated: String = first_line.chars().take(max_chars).collect();
        if first_line.chars().count() > max_chars { format!("{}…", truncated) } else { truncated }
    }
}

pub fn render_sql_editor(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    editor_idx: usize,
    is_focused: bool,
    title: &str,
    theme: &Theme,
) {
    let block = Block::default().title(title).style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height as usize;

    let editor = &state.editor_splits[editor_idx].active_editor();
    let buffer = &editor.buffer;
    let total_lines = buffer.lines.len();

    let show_cursor = matches!(state.mode, Mode::Insert | Mode::Normal | Mode::Visual)
        && state.focused_window == Window::QueryEditor
        && is_focused;

    let scroll = buffer.scroll_offset;
    let end = (scroll + viewport_height).min(total_lines);

    let highlight_cache: Vec<Vec<Span<'static>>> = editor.highlight_lines();

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for row in scroll..end {
        let line_str = &buffer.lines[row];
        let spans =
            highlight_cache.get(row).cloned().unwrap_or_else(|| vec![Span::raw(line_str.clone())]);
        if show_cursor && row == buffer.cursor_row {
            lines.push(editor::render_line_with_cursor(line_str, buffer.cursor_col, spans));
        } else {
            lines.push(Line::from(spans));
        }
    }

    if total_lines > viewport_height && end < total_lines {
        lines.push(Line::from(Span::styled("↓ more...", Style::default().fg(Color::DarkGray))));
    }

    f.render_widget(Paragraph::new(Text::from(lines)).block(block), area);
}
