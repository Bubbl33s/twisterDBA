use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::keymap_help;
use crate::state::{AppState, CellPopupState, Mode, Window};
use crate::ui::utils::{centered_rect, format_count, render_help_footer};

pub fn render_cell_popup(f: &mut Frame, area: Rect, popup: &CellPopupState) {
    let popup_area = centered_rect(80, 80, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Cell: {} ", popup.col_name))
        .style(Style::default().bg(Color::Black));

    let inner_inner = block.inner(popup_area);
    let viewport_height = inner_inner.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = popup
        .value
        .split('\n')
        .skip(popup.scroll)
        .take(viewport_height)
        .map(|s| Line::from(Span::raw(s.to_string())))
        .collect();

    let help_spans = vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("Esc/q/Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ];

    let content = Paragraph::new(Text::from(lines));

    f.render_widget(content.block(block), popup_area);
    render_help_footer(f, popup_area, help_spans);
}

#[allow(clippy::too_many_arguments)]
pub fn render_quick_doc(
    f: &mut Frame,
    area: Rect,
    schema: &str,
    table: &str,
    ddl: Option<&str>,
    row_count: Option<u64>,
    table_size: Option<&str>,
    loading: bool,
    scroll: usize,
    state: &AppState,
) {
    let popup_area = centered_rect(70, 60, area);

    let title =
        format!(" {} — {}.{} ", if loading { "Loading..." } else { "Quick Doc" }, schema, table);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    let viewport_height = inner.height.saturating_sub(2) as usize;

    let mut all_lines: Vec<Line> = Vec::new();

    if let Some(rc) = row_count {
        all_lines.push(Line::from(Span::styled(
            format!(" Row count: {}", format_count(rc)),
            Style::default().fg(Color::Cyan),
        )));
    }
    if let Some(ts) = table_size {
        all_lines.push(Line::from(Span::styled(
            format!(" Table size: {}", ts),
            Style::default().fg(Color::Cyan),
        )));
    }
    if row_count.is_some() || table_size.is_some() {
        all_lines.push(Line::from(""));
    }

    if loading {
        let spinner = state.spinner_char();
        all_lines.push(Line::from(Span::styled(
            format!(" {} Loading table info...", spinner),
            Style::default().fg(Color::Yellow),
        )));
    } else if let Some(ddl_str) = ddl {
        for ddl_line in ddl_str.lines() {
            all_lines.push(Line::from(Span::raw(ddl_line.to_string())));
        }
    } else {
        all_lines.push(Line::from(Span::raw("(no DDL available)")));
    }

    let scroll_top = scroll;
    let end = (scroll_top + viewport_height).min(all_lines.len());
    let visible: Vec<Line> =
        all_lines.into_iter().skip(scroll_top).take(end - scroll_top).collect();

    let help_spans = vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("g/G", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":top/bottom  "),
        Span::styled("Esc/q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ];

    let content = Paragraph::new(Text::from(visible));
    f.render_widget(content.block(block), popup_area);
    render_help_footer(f, popup_area, help_spans);
}

pub fn render_keymap_help(f: &mut Frame, area: Rect, window: &Window, mode: &Mode, scroll: usize) {
    let popup_area = centered_rect(60, 60, area);

    let mode_str = match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Command { .. } => "COMMAND",
        Mode::ConnectDialog { .. } => "CONNECT",
        Mode::Visual => "VISUAL",
    };
    let panel_str = match window {
        Window::SchemaExplorer => "Schema Explorer",
        Window::QueryEditor => "Query Editor",
        Window::OutputResults => "Output / Results",
    };
    let title = format!(" Keymap Help — {} [{}] ", panel_str, mode_str);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    let viewport_height = inner.height.saturating_sub(4) as usize;

    let mut all_lines: Vec<Line> = Vec::new();

    let bindings = keymap_help::get_keybindings(window, mode);
    for binding in bindings {
        let line = Line::from(vec![
            Span::styled(
                format!(" {:>14} ", binding.keys),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(binding.description),
        ]);
        all_lines.push(line);
    }

    let scroll_top = scroll;
    let end = (scroll_top + viewport_height).min(all_lines.len());
    let visible: Vec<Line> =
        all_lines.into_iter().skip(scroll_top).take(end - scroll_top).collect();

    let help_spans = vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("Esc/q/?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ];

    let content = Paragraph::new(Text::from(visible));
    f.render_widget(content.block(block), popup_area);
    render_help_footer(f, popup_area, help_spans);
}
