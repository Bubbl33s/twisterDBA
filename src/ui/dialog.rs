use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::{ConnectForm, DialogStep};
use crate::ui::utils::centered_rect;

const ENGINE_NAMES: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
const ENGINE_ICONS: &[&str] = &["\u{F06FC}", "\u{F07C0}", "\u{F021A}"];
const ENGINE_COLORS: &[Color] =
    &[Color::Rgb(77, 182, 172), Color::Rgb(84, 138, 247), Color::Rgb(169, 183, 198)];

pub fn render_connect_dialog(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
) {
    match form.step {
        DialogStep::SelectType => render_step1(f, area, form, profiles),
        DialogStep::EnterDetails => render_step2(f, area, form),
    }
}

fn render_step1(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
) {
    let popup_area = centered_rect(70, 55, area);

    let title = " New Connection ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);

    let profile_rows: u16 = if profiles.is_empty() { 1 } else { (profiles.len() as u16).min(6) };
    let type_grid_height: u16 = 3;
    let separator_height: u16 = 1;
    let help_height: u16 = 1;

    let available = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(type_grid_height),
            Constraint::Length(separator_height),
            Constraint::Length(profile_rows),
            Constraint::Min(0),
            Constraint::Length(help_height),
        ])
        .split(inner);

    let _spacer = available[0];
    let type_area = available[1];
    let _sep = available[2];
    let profile_area = available[3];
    let _filler = available[4];
    let help_area = available[5];

    render_type_grid(f, type_area, form);
    render_profile_list(f, profile_area, form, profiles);
    render_step1_help(f, help_area);

    f.render_widget(block, popup_area);
}

fn render_type_grid(f: &mut Frame, area: Rect, form: &ConnectForm) {
    let col_width = area.width / 3;
    let mut lines: Vec<Line> = Vec::new();

    let mut icon_line = Vec::new();
    let mut name_line = Vec::new();

    for i in 0..3 {
        let is_selected = form.type_cursor == i && form.selected_profile.is_none();
        let bg = if is_selected { ENGINE_COLORS[i] } else { Color::Black };
        let fg = if is_selected { Color::Black } else { ENGINE_COLORS[i] };

        let icon = ENGINE_ICONS[i];
        let name = ENGINE_NAMES[i];

        let pad = if col_width > 10 { (col_width as usize - 10) / 2 } else { 0 };
        let pad_str = " ".repeat(pad);

        icon_line
            .push(Span::styled(format!("{}{}  ", pad_str, icon), Style::default().fg(fg).bg(bg)));
        icon_line.push(Span::styled(
            format!("{}{}", " ".repeat(col_width as usize - pad - 3), ""),
            Style::default().bg(bg),
        ));

        name_line.push(Span::styled(
            format!("{}{}  ", pad_str, name),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        ));
        name_line.push(Span::styled(
            format!("{}{}", " ".repeat(col_width as usize - pad - 3), ""),
            Style::default().bg(bg),
        ));
    }

    lines.push(Line::from(icon_line));
    lines.push(Line::from(name_line));
    lines.push(Line::from(""));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_profile_list(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " Saved Connections:",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
    )));

    if profiles.is_empty() {
        lines.push(Line::from(Span::styled(
            "   (no saved connections)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, p) in profiles.iter().enumerate() {
            let is_selected = form.selected_profile == Some(i);
            let engine_idx = match p.db_type.as_str() {
                "mysql" => 1,
                "sqlite" => 2,
                _ => 0,
            };
            let icon = ENGINE_ICONS[engine_idx];
            let icon_color = ENGINE_COLORS[engine_idx];

            let bg = if is_selected { Color::Rgb(60, 63, 65) } else { Color::Black };
            let name_style = if is_selected {
                Style::default().fg(Color::White).bg(bg)
            } else {
                Style::default().fg(Color::Gray).bg(bg)
            };

            let line_text = format!("  {} {} ({})", icon, p.name, p.host);
            let mut spans = vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(icon_color).bg(bg)),
                Span::styled(format!("{} ", p.name), name_style),
                Span::styled(format!("({})", p.host), Style::default().fg(Color::DarkGray).bg(bg)),
            ];

            let remaining = area.width as usize;
            let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
            if current_len < remaining {
                spans.push(Span::styled(
                    " ".repeat(remaining - current_len),
                    Style::default().bg(bg),
                ));
            }

            let _ = line_text;
            lines.push(Line::from(spans));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_step1_help(f: &mut Frame, area: Rect) {
    let help_spans = vec![
        Span::styled("←→↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":select  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":cancel"),
    ];
    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(Color::Black));
    f.render_widget(help_paragraph, area);
}

fn render_step2(f: &mut Frame, area: Rect, form: &ConnectForm) {
    let popup_area = centered_rect(60, 60, area);

    let title = " New Connection ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);

    let label_width: u16 = 12;
    let has_ssl = form.db_type == 0;
    let field_count = 1 + form.fields.len() + if has_ssl { 1 } else { 0 };
    let field_height = field_count as u16 + 1;
    let warning_height: u16 = if form.name_conflict { 2 } else { 0 };
    let help_height: u16 = 1;

    let available = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(field_height),
            Constraint::Length(warning_height),
            Constraint::Min(0),
            Constraint::Length(help_height),
        ])
        .split(inner);

    let _spacer = available[0];
    let form_area = available[1];
    let warning_area = if form.name_conflict { Some(available[2]) } else { None };
    let _filler = available[3];
    let help_area = available[4];

    render_step2_fields(f, form_area, form, label_width);

    if let Some(warn_area) = warning_area {
        let name = &form.connection_name;
        let warn_spans = vec![
            Span::styled(" ⚠ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "Connection '{}' already exists. Enter to overwrite, Esc to edit name.",
                    name
                ),
                Style::default().fg(Color::Yellow),
            ),
        ];
        f.render_widget(
            Paragraph::new(Line::from(warn_spans)).style(Style::default().bg(Color::Black)),
            warn_area,
        );
    }

    render_step2_help(f, help_area, form);

    f.render_widget(block, popup_area);
}

fn render_step2_fields(f: &mut Frame, area: Rect, form: &ConnectForm, label_width: u16) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(render_editable_field(
        "Name",
        &form.connection_name,
        form.connection_name_cursor,
        false,
        false,
        form.active_field == 0,
        label_width,
        area.width,
        None,
    ));

    for (i, field) in form.fields.iter().enumerate() {
        let field_idx = i + 1;
        let is_active = form.active_field == field_idx;
        let display_label = if form.db_type == 2 && i == 0 { "File Path" } else { field.label };
        lines.push(render_editable_field(
            display_label,
            &field.value,
            field.cursor,
            field.masked,
            field.keychain_loaded,
            is_active,
            label_width,
            area.width,
            None,
        ));
    }

    if form.db_type == 0 {
        let ssl_idx = form.fields.len() + 1;
        let is_active = form.active_field == ssl_idx;
        let ssl_value = crate::state::ConnectForm::SSL_MODES[form.ssl_mode];
        let ssl_display = format!("{} ←→", ssl_value);
        lines.push(render_editable_field(
            "SSL Mode",
            &ssl_display,
            ssl_display.len(),
            false,
            false,
            is_active,
            label_width,
            area.width,
            Some(ssl_value),
        ));
    }

    f.render_widget(Paragraph::new(lines), area);
}

#[allow(clippy::too_many_arguments)]
fn render_editable_field(
    label: &str,
    value: &str,
    cursor: usize,
    masked: bool,
    keychain_loaded: bool,
    is_active: bool,
    label_width: u16,
    total_width: u16,
    _ssl_value: Option<&str>,
) -> Line<'static> {
    let label_str =
        format!(" {:>width$} ", label, width = (label_width as usize).saturating_sub(2));
    let label_span = Span::styled(
        label_str,
        if is_active {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        },
    );

    let display_value = if masked { "•".repeat(value.len()) } else { value.to_string() };

    let keychain_note = if keychain_loaded { " [from keychain]" } else { "" };

    let field_bg = if is_active { Color::DarkGray } else { Color::Black };
    let field_fg = if is_active { Color::White } else { Color::Gray };

    let mut field_spans: Vec<Span> = Vec::new();

    if is_active && display_value.is_empty() {
        field_spans.push(Span::styled("█", Style::default().fg(Color::Yellow).bg(field_bg)));
    } else {
        for (ci, ch) in display_value.chars().enumerate() {
            if is_active && ci == cursor && ci < display_value.len() {
                field_spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ));
            } else {
                field_spans
                    .push(Span::styled(ch.to_string(), Style::default().fg(field_fg).bg(field_bg)));
            }
        }
        if is_active && cursor >= display_value.len() {
            field_spans.push(Span::styled(" ", Style::default().fg(Color::Black).bg(Color::Cyan)));
        }
    }

    let remaining = total_width.saturating_sub(label_width + 1) as usize;
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
    Line::from(spans)
}

fn render_step2_help(f: &mut Frame, area: Rect, form: &ConnectForm) {
    let mut help_spans = vec![
        Span::styled("Tab/↓↑", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":next  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":back  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":connect"),
    ];
    if form.db_type == 0 {
        help_spans.push(Span::raw("  "));
        help_spans.push(Span::styled(
            "←→",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        help_spans.push(Span::raw(":ssl-mode"));
    }
    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(Color::Black));
    f.render_widget(help_paragraph, area);
}
