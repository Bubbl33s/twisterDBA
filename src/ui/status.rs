use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::state::{AppState, Mode, Window};
use crate::theme::Theme;

pub fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let is_explorer_focused = state.focused_window == Window::SchemaExplorer;
    let is_editor_focused = state.focused_window == Window::QueryEditor;
    let is_results_focused = state.focused_window == Window::OutputResults;

    let explorer_bg =
        if is_explorer_focused { theme.statusline_active_bg } else { theme.statusline_inactive_bg };
    let editor_bg =
        if is_editor_focused { theme.statusline_active_bg } else { theme.statusline_inactive_bg };
    let results_bg =
        if is_results_focused { theme.statusline_active_bg } else { theme.statusline_inactive_bg };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(2),
            Constraint::Percentage(45),
            Constraint::Percentage(30),
        ])
        .split(area);

    let mode_text = match &state.mode {
        Mode::Normal => "NORMAL".to_string(),
        Mode::Insert => "INSERT".to_string(),
        Mode::Command { buffer } => {
            if buffer.is_empty() {
                "COMMAND".to_string()
            } else {
                format!("COMMAND :{}", buffer)
            }
        },
        Mode::ConnectDialog { .. } => "CONNECT".to_string(),
        Mode::Visual => "VISUAL".to_string(),
    };

    let mode_style = match &state.mode {
        Mode::Normal => Style::default().bg(Color::DarkGray).fg(Color::White),
        Mode::Insert => Style::default().bg(Color::Green).fg(Color::Black),
        Mode::Command { .. } => Style::default().bg(Color::Blue).fg(Color::White),
        Mode::ConnectDialog { .. } => Style::default().bg(Color::Cyan).fg(Color::Black),
        Mode::Visual => Style::default().bg(Color::LightMagenta).fg(Color::Black),
    };

    let mut exp_labels: Vec<Span> = Vec::new();
    exp_labels.push(Span::styled(" ", Style::default().bg(explorer_bg)));
    exp_labels
        .push(Span::styled(format!(" {} ", mode_text), mode_style.add_modifier(Modifier::BOLD)));

    let conn_text = render_connection_status(state);
    exp_labels.push(Span::styled(" ", Style::default().bg(explorer_bg)));
    let conn_spans = conn_text.into_iter().collect::<Vec<Span>>();
    exp_labels.extend(conn_spans);

    let keybindings = match &state.mode {
        Mode::Normal => match state.focused_window {
            Window::SchemaExplorer => {
                " j/k:Nav  l:Expand  h:Clps  Tab:Tab  Ctrl+l:Editor  Ctrl+j:Output  R:Reload"
            },
            Window::QueryEditor => {
                if state.focused_editor().executing {
                    " Ctrl+C:Cancel  Ctrl+E:—  Tab:Next Tab  S-Tab:Prev Tab  Space+b:Nuevo"
                } else {
                    " h/j/k/l:Move  w/b/e:Word  i:Edit  Ctrl+E:Exec  Tab:Tab  Space+x:Cerrar"
                }
            },
            Window::OutputResults => {
                if state.output_results.active_tab == 0 {
                    " j/k:Scroll  g/G:Top/Bot  Tab:Results  S-Tab:Results"
                } else if state.active_result_grid().visual_mode {
                    " j/k/h/l:Select  y:Copy  Esc:Exit"
                } else if state.active_result_grid().is_editing() {
                    " Type:Edit  Enter:Commit  Esc:Cancel  ←→:Cursor"
                } else {
                    " j/k:Row  h/l:Col  e:Edit  Enter:Popup  y:Copy  v:Visual  Tab:Next  Space+x:Close"
                }
            },
        },
        Mode::Insert => {
            if state.focused_window == Window::QueryEditor {
                " Esc:Normal  Ctrl+E:Exec  Ctrl+C:Cancel"
            } else {
                " Esc:Normal"
            }
        },
        Mode::Command { .. } => " Enter:Exec  Esc:Cancel",
        Mode::ConnectDialog { .. } => " Tab:Next  Esc:Cancel  Enter:OK",
        Mode::Visual => " Esc:Normal",
    };

    let exp_para = Paragraph::new(Line::from(exp_labels)).style(Style::default().bg(explorer_bg));

    let editor_para = Paragraph::new(Text::from("")).style(Style::default().bg(editor_bg));

    let results_para = Paragraph::new(Text::from(format!(" {}", keybindings)))
        .style(Style::default().bg(results_bg));

    f.render_widget(exp_para, chunks[0]);
    f.render_widget(editor_para, chunks[2]);
    f.render_widget(results_para, chunks[3]);
}

fn render_connection_status(state: &AppState) -> Line<'static> {
    if state.focused_editor().executing {
        let spinner = state.spinner_char();
        return Line::from(Span::styled(
            format!(
                "{} Running... | {} rows",
                spinner,
                state.active_result_grid().total_rows_received
            ),
            Style::default().fg(Color::Yellow),
        ));
    }

    let source_count = state.explorer.sources.len();
    let connected_count = state
        .explorer
        .sources
        .iter()
        .filter(|s| matches!(s.status, crate::state::ConnectionStatus::Connected { .. }))
        .count();

    match state.active_source() {
        None => {
            if source_count > 1 {
                Line::from(Span::styled(
                    format!("{} connections", connected_count),
                    Style::default().fg(Color::Gray),
                ))
            } else {
                Line::from(Span::styled("○ Disconnected", Style::default().fg(Color::Gray)))
            }
        },
        Some(source) => {
            use crate::state::ConnectionStatus;
            let engine_prefix = match source.engine_type {
                crate::db::backend::EngineType::Postgres => "PG",
                crate::db::backend::EngineType::Mysql => "MY",
                crate::db::backend::EngineType::Sqlite => "SQ",
            };
            let count_suffix = if connected_count > 1 {
                format!(" (+{})", connected_count - 1)
            } else {
                String::new()
            };
            match &source.status {
                ConnectionStatus::Connecting { .. } => {
                    let spinner = state.spinner_char();
                    Line::from(Span::styled(
                        format!("{} [{}] {}{}", spinner, engine_prefix, source.name, count_suffix),
                        Style::default().fg(Color::Yellow),
                    ))
                },
                ConnectionStatus::Connected { .. } => Line::from(vec![
                    Span::styled("●", Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", engine_prefix),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" "),
                    Span::styled(source.name.clone(), Style::default().fg(Color::Green)),
                    Span::styled(count_suffix, Style::default().fg(Color::DarkGray)),
                ]),
                ConnectionStatus::Error(msg) => Line::from(Span::styled(
                    format!("[{}] {} ✗ {}", engine_prefix, source.name, msg),
                    Style::default().fg(Color::Red),
                )),
                ConnectionStatus::Disconnected => Line::from(vec![
                    Span::styled("○", Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", engine_prefix),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" "),
                    Span::styled(source.name.clone(), Style::default().fg(Color::Gray)),
                ]),
            }
        },
    }
}
