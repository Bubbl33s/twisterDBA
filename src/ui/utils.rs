use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_dialog_backdrop(f: &mut Frame, area: Rect) {
    use ratatui::widgets::Clear;
    f.render_widget(Clear, area);
    let backdrop =
        Paragraph::new(Text::from("")).style(Style::default().bg(Color::Rgb(20, 20, 20)));
    f.render_widget(backdrop, area);
}

pub fn format_count(n: u64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let mut s = String::new();
    let n_str = n.to_string();
    let len = n_str.len();
    for (i, c) in n_str.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            s.push(',');
        }
        s.push(c);
    }
    s
}

pub fn render_help_footer(f: &mut Frame, popup_area: Rect, help_spans: Vec<Span<'static>>) {
    let help_paragraph =
        Paragraph::new(Line::from(help_spans)).style(Style::default().bg(Color::Black));
    let help_rect =
        Rect { y: popup_area.y + popup_area.height.saturating_sub(3), height: 1, ..popup_area };
    f.render_widget(help_paragraph, help_rect);
}
