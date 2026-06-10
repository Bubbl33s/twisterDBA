use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::ConnectForm;
use crate::ui::utils::centered_rect;

pub fn render_connect_dialog(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
) {
    let popup_area = centered_rect(60, 60, area);

    let db_types = ["PostgreSQL", "MySQL", "SQLite"];
    let title = format!(" Connect to {} ", db_types[form.db_type]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);

    let label_width = 10;

    let type_line_height: u16 = 2;
    let field_count = if form.db_type == 2 { 1 } else { 5 };
    let profile_rows: u16 = (profiles.len() as u16).min(4);
    let field_height = field_count as u16 + 2;
    let total_height = type_line_height + profile_rows + field_height + 2;

    let available = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(((inner.height as i32 - total_height as i32).max(0)) as u16),
            Constraint::Length(type_line_height),
            Constraint::Length(profile_rows),
            Constraint::Length(field_height),
            Constraint::Min(2),
        ])
        .split(inner);

    let _type_area = available[1];
    let _profile_area = available[2];
    let _form_area = available[3];
    let help_area = available[4];

    let mut lines: Vec<Line> = Vec::new();

    {
        let type_is_active = form.selecting_type;
        let type_label = Span::styled(
            " DB Type: ",
            if type_is_active {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        );
        let type_value = Span::styled(
            format!(" <{}> ", db_types[form.db_type]),
            if type_is_active {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            },
        );
        let hint = Span::styled(" ← → to change", Style::default().fg(Color::DarkGray));
        lines.push(Line::from(vec![type_label, type_value, hint]));
    }

    if !profiles.is_empty() {
        let profile_texts: Vec<String> = profiles
            .iter()
            .map(|p| format!(" {} ({}) @ {}:{}/{}", p.name, p.db_type, p.host, p.port, p.database))
            .collect();
        for pt in &profile_texts {
            lines.push(Line::from(Span::styled(pt.clone(), Style::default().fg(Color::DarkGray))));
        }
    }

    let fields_iter: Vec<&crate::state::ConnectField> =
        if form.db_type == 2 { vec![&form.fields[0]] } else { form.fields.iter().collect() };

    for (i, field) in fields_iter.iter().enumerate() {
        let is_active = i == form.active_field;

        let display_label = if form.db_type == 2 { "File Path" } else { field.label };

        let label = format!(" {:>width$} ", display_label, width = label_width as usize - 2);
        let label_span = Span::styled(
            label,
            if is_active {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        );

        let display_value =
            if field.masked { "•".repeat(field.value.len()) } else { field.value.clone() };

        let keychain_note = if field.keychain_loaded { " [from keychain]" } else { "" };

        let field_bg = if is_active { Color::DarkGray } else { Color::Black };
        let field_fg = if is_active { Color::White } else { Color::Gray };

        let mut field_spans: Vec<Span> = Vec::new();

        if is_active && display_value.is_empty() {
            field_spans.push(Span::styled("█", Style::default().fg(Color::Yellow).bg(field_bg)));
        } else {
            for (ci, ch) in display_value.chars().enumerate() {
                if is_active && ci == field.cursor && ci < display_value.len() {
                    field_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::Black).bg(Color::Cyan),
                    ));
                } else {
                    field_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(field_fg).bg(field_bg),
                    ));
                }
            }
            if is_active && field.cursor >= display_value.len() {
                field_spans
                    .push(Span::styled(" ", Style::default().fg(Color::Black).bg(Color::Cyan)));
            }
        }

        let remaining = inner.width.saturating_sub(label_width as u16 + 1) as usize;
        let current_len: usize = field_spans.iter().map(|s| s.content.len()).sum();
        if current_len < remaining {
            let fill = remaining - current_len;
            field_spans.push(Span::styled(" ".repeat(fill), Style::default().bg(field_bg)));
        }

        let mut spans = vec![label_span, Span::raw(" ")];
        spans.extend(field_spans);
        if !keychain_note.is_empty() {
            spans.push(Span::styled(keychain_note, Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines).block(block), popup_area);

    let help_spans = vec![
        Span::styled("Tab/↓↑", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":next  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":cancel  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":connect  "),
        Span::styled("←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":db-type"),
    ];

    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(Color::Black));
    f.render_widget(help_paragraph, help_area);
}
