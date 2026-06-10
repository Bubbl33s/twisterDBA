use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;

pub fn render_line_with_cursor(
    line: &str,
    cursor_col: usize,
    spans: Vec<Span<'static>>,
) -> Line<'static> {
    if (line.is_empty() && cursor_col == 0) || spans.is_empty() {
        return if cursor_col == 0 && line.is_empty() {
            Line::from(vec![Span::styled(
                " ",
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
            )])
        } else {
            Line::from(spans)
        };
    }

    let mut char_count: usize = 0;
    let mut new_spans: Vec<Span<'static>> = Vec::new();
    let mut found = false;

    for span in spans {
        if found {
            new_spans.push(span);
            continue;
        }

        let span_chars: Vec<&str> = span.content.graphemes(true).collect();
        let span_len = span_chars.len();

        if char_count + span_len > cursor_col {
            let cursor_offset = cursor_col - char_count;

            let before: String = span_chars[..cursor_offset].join("");
            let cursor_char = span_chars[cursor_offset].to_string();
            let after: String = span_chars[cursor_offset + 1..].join("");

            if !before.is_empty() {
                new_spans.push(Span::styled(before, span.style));
            }
            new_spans.push(Span::styled(
                cursor_char,
                span.style.bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
            ));
            if !after.is_empty() {
                new_spans.push(Span::styled(after, span.style));
            }

            found = true;
        } else {
            new_spans.push(span);
            char_count += span_len;
        }
    }

    if !found && cursor_col >= char_count {
        new_spans.push(Span::styled(
            " ",
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::REVERSED),
        ));
    }

    Line::from(new_spans)
}
