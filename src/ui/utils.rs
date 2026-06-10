use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
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

pub fn centered_rect_bounded(
    percent_x: u16,
    percent_y: u16,
    max_width: u16,
    max_height: u16,
    r: Rect,
) -> Rect {
    let mut rect = centered_rect(percent_x, percent_y, r);
    if rect.width > max_width {
        let excess = rect.width - max_width;
        rect.x += excess / 2;
        rect.width = max_width;
    }
    if rect.height > max_height {
        let excess = rect.height - max_height;
        rect.y += excess / 2;
        rect.height = max_height;
    }
    rect
}

pub fn render_dialog_backdrop(_f: &mut Frame, _area: Rect) {}

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
