use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::explorer::{NodeKind, SchemaExplorer};
use crate::state::{AppState, ConnectionStatus, Window};
use crate::theme::Theme;

pub fn render_database_explorer(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    explorer: &SchemaExplorer,
    theme: &Theme,
) {
    let is_focused = state.focused_window == Window::SchemaExplorer;
    let title_style = if is_focused {
        Style::default().bg(theme.statusline_active_bg)
    } else {
        Style::default().bg(theme.statusline_inactive_bg)
    };
    let explorer_block = Block::default()
        .title(Span::styled(" Database Explorer ", title_style.add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(theme.bg));

    if explorer.sources.is_empty() {
        f.render_widget(Paragraph::new(Text::from("(no connections)")).block(explorer_block), area);
        return;
    }

    if explorer.search_active && explorer.flat_view.is_empty() {
        f.render_widget(Paragraph::new(Text::from("(no matches)")).block(explorer_block), area);
        return;
    }

    let nerd_font_available = theme.nerd_font_available;

    let items: Vec<ListItem> = explorer
        .flat_view
        .iter()
        .map(|node| {
            let indent = "  ".repeat(node.depth);
            let indicator =
                if node.expandable { if node.expanded { "▾ " } else { "▸ " } } else { "  " };
            let indicator_span = Span::styled(indicator, Style::default().fg(Color::DarkGray));

            let icon_span = if let Some((icon_char, icon_color)) = &node.icon {
                if nerd_font_available {
                    Span::styled(format!("{} ", icon_char), Style::default().fg(*icon_color))
                } else {
                    let ascii = match node.kind {
                        NodeKind::Source => {
                            let src = explorer.sources.iter().find(|s| s.name == node.name);
                            match src.map(|s| s.engine_type) {
                                Some(crate::db::backend::EngineType::Postgres) => "[PG] ",
                                Some(crate::db::backend::EngineType::Mysql) => "[MY] ",
                                Some(crate::db::backend::EngineType::Sqlite) => "[SQ] ",
                                None => "[DB] ",
                            }
                        },
                        NodeKind::Schema => "[S] ",
                        NodeKind::Table => "[T] ",
                        NodeKind::View => "[V] ",
                        _ => "",
                    };
                    Span::styled(ascii.to_string(), Style::default().fg(*icon_color))
                }
            } else {
                Span::raw("")
            };

            let text = match node.kind {
                NodeKind::Column => {
                    let nullable_str = if node.nullable { "" } else { " NOT NULL" };
                    format!(
                        "{} {}{}",
                        node.name,
                        node.data_type.as_deref().unwrap_or(""),
                        nullable_str
                    )
                },
                _ => node.name.clone(),
            };

            let mut spans = vec![Span::raw(indent), indicator_span, icon_span, Span::raw(text)];

            if node.kind == NodeKind::Source
                && let Some(source) = explorer.sources.iter().find(|s| s.name == node.name)
            {
                let status_span = render_status_indicator(&source.status, theme, state);
                spans.push(Span::raw(" "));
                spans.push(status_span);
            }

            ListItem::new(Text::from(Line::from(spans)))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(explorer.selected_idx));

    let highlight_style = if is_focused {
        Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Black).bg(Color::DarkGray).add_modifier(Modifier::BOLD)
    };

    let list =
        List::new(items).block(explorer_block).highlight_style(highlight_style).scroll_padding(2);

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_status_indicator(
    status: &ConnectionStatus,
    theme: &Theme,
    state: &AppState,
) -> Span<'static> {
    match status {
        ConnectionStatus::Connected { .. } => {
            Span::styled("●", Style::default().fg(theme.status_connected))
        },
        ConnectionStatus::Connecting { .. } => {
            let spinner = state.spinner_char();
            Span::styled(spinner.to_string(), Style::default().fg(theme.status_connecting))
        },
        ConnectionStatus::Error(_) => Span::styled("✗", Style::default().fg(theme.status_error)),
        ConnectionStatus::Disconnected => {
            Span::styled("○", Style::default().fg(theme.status_disconnected))
        },
    }
}
