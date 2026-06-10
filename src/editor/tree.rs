use std::collections::HashMap;
use std::ops::Range;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use tree_sitter::{Parser, Tree, TreeCursor};

pub struct TsParser {
    parser: Parser,
}

impl TsParser {
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language: tree_sitter::Language = tree_sitter_sequel::LANGUAGE.into();
        parser.set_language(&language).expect("Failed to set tree-sitter-sequel language");
        Self { parser }
    }

    pub fn parse(&mut self, source: &str) -> Option<Tree> {
        self.parser.parse(source, None)
    }

    pub fn highlight_line(
        &self,
        tree: &Tree,
        source: &str,
        line_str: &str,
        line_number: usize,
    ) -> Vec<Span<'static>> {
        let mut ranges: Vec<(usize, usize, Style)> = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        Self::walk_for_highlight(&mut cursor, source, line_number, &mut ranges);
        if ranges.is_empty() {
            return vec![Span::raw(line_str.to_string())];
        }
        ranges.sort_by_key(|(s, _, _)| *s);
        let line_start = Self::line_start_byte(source, line_number);
        let line_end_excl = line_start + line_str.len();
        let filtered: Vec<(usize, usize, Style)> =
            ranges.into_iter().filter(|(s, e, _)| *e > line_start && *s < line_end_excl).collect();

        let mut result = Vec::new();
        let mut pos: usize = 0;
        for (start, end, style) in &filtered {
            let s = start.saturating_sub(line_start);
            let e = (end.saturating_sub(line_start)).min(line_str.len());
            if s > pos {
                result.push(Span::styled(line_str[pos..s].to_string(), Style::default()));
            }
            if e > s {
                result.push(Span::styled(line_str[s..e].to_string(), *style));
            }
            pos = e;
        }
        if pos < line_str.len() {
            result.push(Span::styled(line_str[pos..].to_string(), Style::default()));
        }
        result
    }

    fn walk_for_highlight(
        cursor: &mut TreeCursor,
        source: &str,
        line_number: usize,
        ranges: &mut Vec<(usize, usize, Style)>,
    ) {
        let node = cursor.node();
        let s = node.start_byte();
        let e = node.end_byte();

        let start_line = Self::byte_to_line(source, s);
        let end_line = Self::byte_to_line(source, e.saturating_sub(1));

        if end_line < line_number {
            if cursor.goto_next_sibling() {
                Self::walk_for_highlight(cursor, source, line_number, ranges);
            }
            return;
        }
        if start_line > line_number {
            return;
        }

        if node.is_named() {
            let kind = node.kind();
            let style = resolve_style(kind);
            if let Some(style) = style
                && (node.child_count() == 0
                    || !node.has_named_descendant_on_line(line_number, source))
            {
                ranges.push((s, e, style));
            }
        }

        if cursor.goto_first_child() {
            Self::walk_for_highlight(cursor, source, line_number, ranges);
            cursor.goto_parent();
        }

        if cursor.goto_next_sibling() {
            Self::walk_for_highlight(cursor, source, line_number, ranges);
        }
    }

    pub fn find_statement_at(&mut self, source: &str, cursor_byte: usize) -> Option<Range<usize>> {
        let tree = self.parse(source)?;
        let root = tree.root_node();
        let mut cursor = root.walk();
        Self::find_stmt_recursive(&mut cursor, cursor_byte)
    }

    fn find_stmt_recursive(cursor: &mut TreeCursor, cursor_byte: usize) -> Option<Range<usize>> {
        let node = cursor.node();
        if node.start_byte() <= cursor_byte && cursor_byte <= node.end_byte() {
            if node.is_named() && is_statement_or_root(&node) {
                let mut range = node.start_byte()..node.end_byte();
                if cursor.goto_next_sibling() {
                    let sibling = cursor.node();
                    if !sibling.is_named() && sibling.kind() == ";" {
                        range.end = sibling.end_byte();
                    }
                    cursor.goto_previous_sibling();
                }
                return Some(range);
            }
            if cursor.goto_first_child() {
                loop {
                    let result = Self::find_stmt_recursive(cursor, cursor_byte);
                    if result.is_some() {
                        cursor.goto_parent();
                        return result;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }
        None
    }

    pub fn extract_text<'a>(&self, source: &'a str, range: Range<usize>) -> &'a str {
        let end = range.end.min(source.len());
        let start = range.start.min(end);
        &source[start..end]
    }

    fn line_start_byte(source: &str, line: usize) -> usize {
        if line == 0 {
            return 0;
        }
        source
            .bytes()
            .enumerate()
            .filter(|(_, b)| *b == b'\n')
            .nth(line - 1)
            .map(|(i, _)| i + 1)
            .unwrap_or(source.len())
    }

    fn byte_to_line(source: &str, byte: usize) -> usize {
        let cap = byte.min(source.len());
        source[..cap].bytes().filter(|b| *b == b'\n').count()
    }
}

fn is_statement_or_root(node: &tree_sitter::Node) -> bool {
    node.kind() == "statement"
}

fn resolve_style(kind: &str) -> Option<Style> {
    if let Some(style) = STYLE_MAP.get(kind) {
        return Some(*style);
    }
    if kind.starts_with("keyword_") {
        return Some(Style::default().fg(Color::Rgb(204, 120, 50)).add_modifier(Modifier::BOLD));
    }
    None
}

static STYLE_MAP: std::sync::LazyLock<HashMap<&'static str, Style>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("literal", Style::default().fg(Color::Rgb(106, 135, 89)));
        m.insert("dollar_quote", Style::default().fg(Color::Rgb(106, 135, 89)));
        m.insert(
            "comment",
            Style::default().fg(Color::Rgb(128, 128, 128)).add_modifier(Modifier::ITALIC),
        );
        m.insert(
            "marginalia",
            Style::default().fg(Color::Rgb(128, 128, 128)).add_modifier(Modifier::ITALIC),
        );
        m.insert("identifier", Style::default().fg(Color::Rgb(169, 183, 198)));
        m.insert("field", Style::default().fg(Color::Rgb(152, 118, 170)));
        m.insert("parameter", Style::default().fg(Color::Rgb(106, 135, 89)));
        m.insert("invocation", Style::default().fg(Color::Rgb(255, 198, 109)));
        m.insert("statement", Style::default());
        m
    });

trait NodeExt {
    fn has_named_descendant_on_line(&self, line: usize, source: &str) -> bool;
}

impl<'a> NodeExt for tree_sitter::Node<'a> {
    fn has_named_descendant_on_line(&self, line: usize, source: &str) -> bool {
        let mut cursor = self.walk();
        if !cursor.goto_first_child() {
            return false;
        }
        loop {
            let child = cursor.node();
            if child.is_named() {
                let start_line = TsParser::byte_to_line(source, child.start_byte());
                let end_line = TsParser::byte_to_line(source, child.end_byte().saturating_sub(1));
                if start_line <= line && line <= end_line {
                    return true;
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stmt_at_start() {
        let mut parser = TsParser::new();
        let source = "SELECT * FROM users;";
        let range = parser.find_statement_at(source, 0);
        assert!(range.is_some());
        let r = range.unwrap();
        assert_eq!(&source[r.start..r.end], source);
    }

    #[test]
    fn test_stmt_in_middle() {
        let mut parser = TsParser::new();
        let source = "SELECT * FROM users WHERE id = 1;";
        let from_pos = source.find("FROM").unwrap();
        let range = parser.find_statement_at(source, from_pos);
        assert!(range.is_some());
        let r = range.unwrap();
        assert_eq!(&source[r.start..r.end], source);
    }

    #[test]
    fn test_stmt_at_end() {
        let mut parser = TsParser::new();
        let source = "SELECT 1;\nSELECT 2;";
        let range = parser.find_statement_at(source, 11);
        assert!(range.is_some(), "Expected to find statement at byte 11 in '{}'", source);
    }

    #[test]
    fn test_between_statements() {
        let mut parser = TsParser::new();
        let source = "SELECT 1;\n\nSELECT 2;";
        let blank_pos = source.find("\n\n").unwrap() + 1;
        let range = parser.find_statement_at(source, blank_pos);
        assert!(range.is_none());
    }

    #[test]
    fn test_in_string() {
        let mut parser = TsParser::new();
        let source = "SELECT 'hello world' AS greeting;";
        let string_pos = source.find("hello").unwrap();
        let range = parser.find_statement_at(source, string_pos);
        assert!(range.is_some());
        let r = range.unwrap();
        assert_eq!(&source[r.start..r.end], source);
    }

    #[test]
    fn test_in_comment() {
        let mut parser = TsParser::new();
        let source = "-- this is a comment\nSELECT 1;";
        let comment_pos = 5; // middle of comment
        let range = parser.find_statement_at(source, comment_pos);
        assert!(range.is_none());
    }

    #[test]
    fn test_emoji_in_string() {
        let mut parser = TsParser::new();
        let source = "SELECT 'hello 🚀 world' AS test;";
        let emoji_pos = source.find("🚀").unwrap();
        let range = parser.find_statement_at(source, emoji_pos);
        assert!(range.is_some());
        let r = range.unwrap();
        let extracted = &source[r.start..r.end];
        assert!(extracted.contains("🚀"));
        assert!(!extracted.is_empty());
    }

    #[test]
    fn test_cjk_in_comment() {
        let mut parser = TsParser::new();
        let source = "-- こんにちは世界\nSELECT 1;";
        let comment_pos = 5;
        let range = parser.find_statement_at(source, comment_pos);
        assert!(range.is_none());
    }

    #[test]
    fn test_extract_text() {
        let parser = TsParser::new();
        let source = "SELECT 1;";
        let extracted = parser.extract_text(source, 0..9);
        assert_eq!(extracted, "SELECT 1;");
    }

    #[test]
    fn test_extract_text_oob() {
        let parser = TsParser::new();
        let source = "SELECT 1;";
        let extracted = parser.extract_text(source, 0..999);
        assert_eq!(extracted, "SELECT 1;");
    }
}
