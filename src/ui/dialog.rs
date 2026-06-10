use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::{ConnectForm, DialogStep};
use crate::theme::Theme;
use crate::ui::utils::centered_rect_bounded;

const ENGINE_NAMES: &[&str] = &["PostgreSQL", "MySQL", "SQLite"];
const ENGINE_ICONS: &[&str] = &["\u{F06FC}", "\u{F07C0}", "\u{F021A}"];

pub fn render_connect_dialog(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
    theme: &Theme,
) {
    match form.step {
        DialogStep::SelectType => render_step1(f, area, form, profiles, theme),
        DialogStep::EnterDetails => render_step2(f, area, form, theme),
    }
}

fn render_step1(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
    theme: &Theme,
) {
    let profile_count = profiles.len();
    let content_height: u16 = {
        let engines: u16 = ConnectForm::ENGINE_COUNT as u16;
        let separator: u16 = if profile_count > 0 { 2 } else { 0 };
        let prof_rows: u16 = profile_count as u16;
        let help: u16 = 1;
        let padding: u16 = 2;
        engines + separator + prof_rows + help + padding
    };

    let popup_area = centered_rect_bounded(80, 70, 50, content_height.max(12), area);

    let title = " New Connection ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme.identifier))
        .style(Style::default().bg(theme.editor_bg));

    let inner = block.inner(popup_area);

    let engines_height = ConnectForm::ENGINE_COUNT as u16;
    let separator_height: u16 = if profile_count > 0 { 2 } else { 0 };
    let profile_rows: u16 = profile_count as u16;
    let help_height: u16 = 1;

    let constraints = if profile_count > 0 {
        vec![
            Constraint::Length(1),
            Constraint::Length(engines_height),
            Constraint::Length(separator_height),
            Constraint::Length(profile_rows),
            Constraint::Min(0),
            Constraint::Length(help_height),
        ]
    } else {
        vec![
            Constraint::Length(1),
            Constraint::Length(engines_height),
            Constraint::Min(0),
            Constraint::Length(help_height),
        ]
    };

    let available =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(inner);

    let mut idx = 0;
    let _spacer = available[idx];
    idx += 1;
    let engine_area = available[idx];
    idx += 1;

    if profile_count > 0 {
        let sep_area = available[idx];
        idx += 1;
        let sep_line = Paragraph::new(Line::from(Span::styled(
            " ──────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )))
        .style(Style::default().bg(theme.editor_bg));
        f.render_widget(sep_line, sep_area);

        let profile_area = available[idx];
        render_profile_list_vertical(f, engine_area, profile_area, form, profiles, theme);
    } else {
        render_engine_list(f, engine_area, form, theme);
    }

    let help_area = available[available.len() - 1];
    render_step1_help(f, help_area, theme);

    f.render_widget(block, popup_area);
}

fn render_engine_list(f: &mut Frame, area: Rect, form: &ConnectForm, theme: &Theme) {
    let mut lines: Vec<Line> = Vec::new();

    for i in 0..ConnectForm::ENGINE_COUNT {
        let is_selected = form.cursor_position == i;
        let icon = ENGINE_ICONS[i];
        let name = ENGINE_NAMES[i];

        let icon_color = match i {
            0 => theme.icons.postgres.1,
            1 => theme.icons.mysql.1,
            _ => theme.icons.sqlite.1,
        };

        let bg = if is_selected { theme.dialog_type_selected_bg } else { theme.editor_bg };
        let fg = if is_selected { Color::White } else { Color::Gray };

        let mut spans = vec![
            Span::styled(
                format!("{} ", if is_selected { ">" } else { " " }),
                Style::default().fg(fg).bg(bg),
            ),
            Span::styled(format!("{} ", icon), Style::default().fg(icon_color).bg(bg)),
            Span::styled(
                name.to_string(),
                Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            ),
        ];

        let remaining = area.width as usize;
        let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if current_len < remaining {
            spans.push(Span::styled(" ".repeat(remaining - current_len), Style::default().bg(bg)));
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_profile_list_vertical(
    f: &mut Frame,
    engine_area: Rect,
    profile_area: Rect,
    form: &ConnectForm,
    profiles: &[crate::config::ConnectionProfile],
    theme: &Theme,
) {
    render_engine_list(f, engine_area, form, theme);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " Saved Connections:",
        Style::default().fg(theme.identifier).add_modifier(Modifier::BOLD).bg(theme.editor_bg),
    )));

    for (i, p) in profiles.iter().enumerate() {
        let cursor_pos = ConnectForm::ENGINE_COUNT + i;
        let is_selected = form.cursor_position == cursor_pos;
        let engine_idx = match p.db_type.as_str() {
            "mysql" => 1,
            "sqlite" => 2,
            _ => 0,
        };
        let icon = ENGINE_ICONS[engine_idx];
        let icon_color = match engine_idx {
            0 => theme.icons.postgres.1,
            1 => theme.icons.mysql.1,
            _ => theme.icons.sqlite.1,
        };

        let bg = if is_selected { theme.dialog_type_selected_bg } else { theme.editor_bg };
        let name_style = if is_selected {
            Style::default().fg(Color::White).bg(bg)
        } else {
            Style::default().fg(Color::Gray).bg(bg)
        };

        let mut spans = vec![
            Span::styled(
                format!("{} ", if is_selected { ">" } else { " " }),
                Style::default().fg(if is_selected { Color::White } else { Color::Gray }).bg(bg),
            ),
            Span::styled(format!("{} ", icon), Style::default().fg(icon_color).bg(bg)),
            Span::styled(format!("{} ", p.name), name_style),
            Span::styled(format!("({})", p.host), Style::default().fg(Color::DarkGray).bg(bg)),
        ];

        let remaining = profile_area.width as usize;
        let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if current_len < remaining {
            spans.push(Span::styled(" ".repeat(remaining - current_len), Style::default().bg(bg)));
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), profile_area);
}

fn render_step1_help(f: &mut Frame, area: Rect, theme: &Theme) {
    let help_spans = vec![
        Span::styled(
            "↑↓/jk",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":navigate  "),
        Span::styled(
            "Enter",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":select  "),
        Span::styled(
            "Esc",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":cancel"),
    ];
    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(theme.editor_bg));
    f.render_widget(help_paragraph, area);
}

fn render_step2(f: &mut Frame, area: Rect, form: &ConnectForm, theme: &Theme) {
    let label_width: u16 = 12;
    let has_ssl = form.db_type == 0;
    let field_count = 1 + form.fields.len() + if has_ssl { 1 } else { 0 };
    let content_height: u16 =
        (field_count as u16) + 1 + if form.name_conflict { 2 } else { 0 } + 1 + 2;

    let popup_area = centered_rect_bounded(80, 70, 60, content_height.max(12), area);

    let title = " New Connection ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme.identifier))
        .style(Style::default().bg(theme.editor_bg));

    let inner = block.inner(popup_area);

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

    render_step2_fields(f, form_area, form, label_width, theme);

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
            Paragraph::new(Line::from(warn_spans)).style(Style::default().bg(theme.editor_bg)),
            warn_area,
        );
    }

    render_step2_help(f, help_area, form, theme);

    f.render_widget(block, popup_area);
}

fn render_step2_fields(
    f: &mut Frame,
    area: Rect,
    form: &ConnectForm,
    label_width: u16,
    theme: &Theme,
) {
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
        theme,
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
            theme,
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
            theme,
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
    theme: &Theme,
) -> Line<'static> {
    let label_str =
        format!(" {:>width$} ", label, width = (label_width as usize).saturating_sub(2));
    let label_span = Span::styled(
        label_str,
        if is_active {
            Style::default().fg(Color::White).bg(theme.dialog_field_active_bg)
        } else {
            Style::default().fg(Color::DarkGray)
        },
    );

    let display_value = if masked { "•".repeat(value.len()) } else { value.to_string() };

    let keychain_note = if keychain_loaded { " [from keychain]" } else { "" };

    let field_bg = if is_active { theme.dialog_field_active_bg } else { theme.editor_bg };
    let field_fg = if is_active { Color::White } else { Color::Gray };

    let mut field_spans: Vec<Span> = Vec::new();

    if is_active && display_value.is_empty() {
        field_spans.push(Span::styled(
            "\u{258C}",
            Style::default().fg(theme.dialog_cursor_bg).bg(theme.dialog_cursor_fg),
        ));
    } else {
        for (ci, ch) in display_value.chars().enumerate() {
            if is_active && ci == cursor && ci < display_value.len() {
                field_spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(theme.dialog_cursor_bg).bg(theme.dialog_cursor_fg),
                ));
            } else {
                field_spans
                    .push(Span::styled(ch.to_string(), Style::default().fg(field_fg).bg(field_bg)));
            }
        }
        if is_active && cursor >= display_value.len() {
            field_spans.push(Span::styled(
                " ",
                Style::default().fg(theme.dialog_cursor_bg).bg(theme.dialog_cursor_fg),
            ));
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

fn render_step2_help(f: &mut Frame, area: Rect, form: &ConnectForm, theme: &Theme) {
    let mut help_spans = vec![
        Span::styled(
            "Tab/↓↑",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":next  "),
        Span::styled(
            "Esc",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":back  "),
        Span::styled(
            "Enter",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ),
        Span::raw(":connect"),
    ];
    if form.db_type == 0 {
        help_spans.push(Span::raw("  "));
        help_spans.push(Span::styled(
            "←→",
            Style::default().fg(theme.statusline_active_bg).add_modifier(Modifier::BOLD),
        ));
        help_spans.push(Span::raw(":ssl-mode"));
    }
    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(theme.editor_bg));
    f.render_widget(help_paragraph, area);
}
