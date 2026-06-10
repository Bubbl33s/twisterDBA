use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState},
};

use crate::editor;
use crate::explorer::{NodeKind, SchemaExplorer};
use crate::keymap_help;
use crate::state::{AppState, ConnectionStatus, Mode, Panel, PopupState, SplitDirection};
use crate::theme::Theme;

pub fn render(f: &mut Frame, state: &AppState) {
    let full_area = f.area();
    let theme = &state.theme;

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(full_area);

    render_main_area(f, main_layout[0], state, theme);
    render_status_bar(f, main_layout[1], state, theme);

    if let Mode::ConnectDialog { form } = &state.mode {
        render_connect_dialog(f, full_area, form, &state.config.connections);
    }

    if let Some(ref popup) = state.cell_popup {
        render_cell_popup(f, full_area, popup);
    }

    match &state.popup {
        PopupState::QuickDoc { schema, table, ddl, row_count, table_size, loading, scroll } => {
            render_quick_doc(
                f,
                full_area,
                schema,
                table,
                ddl.as_deref(),
                *row_count,
                table_size.as_deref(),
                *loading,
                *scroll,
                state,
            );
        },
        PopupState::KeymapHelp { scroll } => {
            render_keymap_help(f, full_area, &state.focused_panel, &state.mode, *scroll);
        },
        PopupState::None => {},
    }
}

fn render_main_area(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(2),
            Constraint::Percentage(75),
        ])
        .split(area);

    render_schema_explorer(f, chunks[0], &state.explorer, theme);

    let editor_area = chunks[2];
    let editor_count = state.editors.len();
    if editor_count == 1 {
        render_single_editor(f, editor_area, state, 0, theme);
    } else {
        match state.split_direction {
            SplitDirection::Vertical => {
                let each_pct = 100 / editor_count as u16;
                let constraints: Vec<Constraint> =
                    (0..editor_count).map(|_| Constraint::Percentage(each_pct)).collect();
                let editor_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(editor_area);
                for (i, ec) in editor_chunks.iter().enumerate() {
                    render_single_editor(f, *ec, state, i, theme);
                    if i + 1 < editor_count {
                        let sep_rect =
                            Rect { x: ec.x + ec.width, y: ec.y, width: 1, height: ec.height };
                        let sep =
                            Paragraph::new(Text::from(" ")).style(Style::default().bg(theme.bg));
                        f.render_widget(sep, sep_rect);
                    }
                }
            },
            SplitDirection::Horizontal => {
                let each_pct = 100 / editor_count as u16;
                let constraints: Vec<Constraint> =
                    (0..editor_count).map(|_| Constraint::Percentage(each_pct)).collect();
                let editor_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(editor_area);
                for (i, ec) in editor_chunks.iter().enumerate() {
                    render_single_editor(f, *ec, state, i, theme);
                    if i + 1 < editor_count {
                        let sep_rect =
                            Rect { x: ec.x, y: ec.y + ec.height, width: ec.width, height: 1 };
                        let sep =
                            Paragraph::new(Text::from(" ")).style(Style::default().bg(theme.bg));
                        f.render_widget(sep, sep_rect);
                    }
                }
            },
        }
    }
}

fn render_single_editor(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    editor_idx: usize,
    theme: &Theme,
) {
    let editor_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let editor = &state.editors[editor_idx];
    let is_focused = editor_idx == state.focused_editor;

    let editor_title = if editor.executing {
        format!(" SQL Editor {} [running...] ", editor_idx + 1)
    } else {
        format!(" SQL Editor {} ", editor_idx + 1)
    };

    render_sql_editor(f, editor_chunks[0], state, editor_idx, is_focused, &editor_title, theme);

    let bottom_area = editor_chunks[1];
    let tab_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(bottom_area);

    let show_output = state.focused_panel == Panel::Output;
    let show_results = state.focused_panel == Panel::ResultGrid;

    render_bottom_tab_bar(f, tab_chunks[0], show_output, show_results, theme);

    let output = &state.editors[editor_idx].output_pane;
    let grid_title = if state.result_grid.is_streaming && is_focused {
        format!(" Results ({} / streaming...) ", state.result_grid.total_rows_received)
    } else {
        format!(" Results ({} rows) ", state.result_grid.total_rows_received)
    };

    if show_output {
        render_output_pane(f, tab_chunks[1], output, theme);
    } else if show_results {
        render_result_grid(f, tab_chunks[1], state, &grid_title, theme);
    } else {
        render_output_pane(f, tab_chunks[1], output, theme);
    }
}

fn render_bottom_tab_bar(
    f: &mut Frame,
    area: Rect,
    show_output: bool,
    show_results: bool,
    theme: &Theme,
) {
    let output_style = if show_output {
        Style::default().fg(Color::Black).bg(Color::White)
    } else {
        Style::default().fg(Color::Gray).bg(theme.editor_bg)
    };
    let results_style = if show_results {
        Style::default().fg(Color::Black).bg(Color::White)
    } else {
        Style::default().fg(Color::Gray).bg(theme.editor_bg)
    };

    let bar = Line::from(vec![
        Span::styled(" Output ", output_style),
        Span::styled(" ", Style::default().bg(theme.editor_bg)),
        Span::styled(" Results ", results_style),
        Span::styled(
            " ".repeat(area.width.saturating_sub(17) as usize),
            Style::default().bg(theme.editor_bg),
        ),
    ]);

    f.render_widget(Paragraph::new(bar), area);
}

fn render_schema_explorer(f: &mut Frame, area: Rect, explorer: &SchemaExplorer, theme: &Theme) {
    let explorer_block =
        Block::default().title(" Schema Explorer ").style(Style::default().bg(theme.bg));

    if explorer.all_flat_nodes.is_empty() {
        f.render_widget(Paragraph::new(Text::from("(no schema)")).block(explorer_block), area);
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
            let first_span = if let Some((icon_char, icon_color)) = &node.icon {
                if nerd_font_available {
                    Span::styled(format!("{} ", icon_char), Style::default().fg(*icon_color))
                } else {
                    let ascii = match node.kind {
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

            let spans = vec![Span::raw(indent), first_span, Span::raw(text)];
            ListItem::new(Text::from(Line::from(spans)))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(explorer.selected_idx));

    let list = List::new(items)
        .block(explorer_block)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .scroll_padding(2);

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_output_pane(
    f: &mut Frame,
    area: Rect,
    output: &crate::state::OutputPaneState,
    theme: &Theme,
) {
    let block = Block::default().title(" Output ").style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height.saturating_sub(1) as usize;

    if output.lines.is_empty() {
        f.render_widget(Paragraph::new(Text::from("(no output)")).block(block), area);
        return;
    }

    let scroll_top = output.scroll.saturating_sub(viewport_height.saturating_sub(1));
    let end = (scroll_top + viewport_height).min(output.lines.len());

    let lines: Vec<Line> = output
        .lines
        .iter()
        .skip(scroll_top)
        .take(end.saturating_sub(scroll_top))
        .map(|s| {
            if s.contains("ERROR") {
                Line::from(Span::styled(s.clone(), Style::default().fg(Color::Red)))
            } else {
                Line::from(Span::raw(s.clone()))
            }
        })
        .collect();

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let is_explorer_focused = state.focused_panel == Panel::SchemaExplorer;
    let is_editor_focused = state.focused_panel == Panel::QueryEditor;
    let is_results_focused = state.focused_panel == Panel::ResultGrid;
    let _is_output_focused = state.focused_panel == Panel::Output;

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
        Mode::Normal => match state.focused_panel {
            Panel::SchemaExplorer => " j/k:Nav  l:Expand  h:Clps  Tab:Editor  R:Reload",
            Panel::QueryEditor => {
                if state.focused_editor().executing {
                    " Ctrl+C:Cancel  Ctrl+E:—  Tab:Results"
                } else {
                    " h/j/k/l:Move  w/b/e:Word  i:Edit  Ctrl+E:Exec  Tab:Results"
                }
            },
            Panel::ResultGrid => {
                if state.result_grid.visual_mode {
                    " j/k/h/l:Select  y:Copy  Esc:Exit"
                } else if state.result_grid.is_editing() {
                    " Type:Edit  Enter:Commit  Esc:Cancel  ←→:Cursor"
                } else {
                    " j/k:Row  h/l:Col  e:Edit  Enter:Popup  y:Copy  v:Visual  Tab:Output"
                }
            },
            Panel::Output => " j/k:Scroll  g/G:Top/Bottom  Tab:Schema",
        },
        Mode::Insert => {
            if state.focused_panel == Panel::QueryEditor {
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
            format!("{} Running... | {} rows", spinner, state.result_grid.total_rows_received),
            Style::default().fg(Color::Yellow),
        ));
    }

    match &state.connection_status {
        ConnectionStatus::Disconnected => {
            Line::from(Span::styled("○ Disconnected", Style::default().fg(Color::Gray)))
        },
        ConnectionStatus::Connecting { .. } => {
            let spinner = state.spinner_char();
            Line::from(Span::styled(
                format!("{} Connecting...", spinner),
                Style::default().fg(Color::Yellow),
            ))
        },
        ConnectionStatus::Connected { masked, .. } => {
            Line::from(Span::styled(format!("● {}", masked), Style::default().fg(Color::Green)))
        },
        ConnectionStatus::Error(msg) => {
            Line::from(Span::styled(format!("✗ {}", msg), Style::default().fg(Color::Red)))
        },
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn render_connect_dialog(
    f: &mut Frame,
    area: Rect,
    form: &crate::state::ConnectForm,
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

fn render_sql_editor(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    editor_idx: usize,
    is_focused: bool,
    title: &str,
    theme: &Theme,
) {
    let block = Block::default().title(title).style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height as usize;

    let editor = &state.editors[editor_idx];
    let buffer = &editor.buffer;
    let total_lines = buffer.lines.len();

    let show_cursor = matches!(state.mode, Mode::Insert | Mode::Normal | Mode::Visual)
        && state.focused_panel == Panel::QueryEditor
        && is_focused;

    let scroll = buffer.scroll_offset;
    let end = (scroll + viewport_height).min(total_lines);

    let highlight_cache: Vec<Vec<Span<'static>>> = editor.highlight_lines();

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for row in scroll..end {
        let line_str = &buffer.lines[row];
        let spans =
            highlight_cache.get(row).cloned().unwrap_or_else(|| vec![Span::raw(line_str.clone())]);
        if show_cursor && row == buffer.cursor_row {
            lines.push(editor::render_line_with_cursor(line_str, buffer.cursor_col, spans));
        } else {
            lines.push(Line::from(spans));
        }
    }

    if total_lines > viewport_height && end < total_lines {
        lines.push(Line::from(Span::styled("↓ more...", Style::default().fg(Color::DarkGray))));
    }

    f.render_widget(Paragraph::new(Text::from(lines)).block(block), area);
}

fn render_result_grid(f: &mut Frame, area: Rect, state: &AppState, title: &str, theme: &Theme) {
    let block = Block::default().title(title).style(Style::default().bg(theme.editor_bg));
    let inner = block.inner(area);
    let viewport_height = inner.height.saturating_sub(2) as usize;
    let viewport_width = inner.width as usize;

    let grid = &state.result_grid;
    grid.scroll.viewport_height.set(viewport_height);
    grid.scroll.viewport_width.set(viewport_width);

    if grid.rows.is_empty() {
        if state.focused_editor().executing {
            let spinner = state.spinner_char();
            f.render_widget(
                Paragraph::new(Text::from(format!("{} Running query...", spinner))).block(block),
                area,
            );
        } else if let Some(ref err) = state.last_query_error {
            f.render_widget(
                Paragraph::new(Text::from(format!("Error: {}", err)))
                    .style(Style::default().fg(Color::Red))
                    .block(block),
                area,
            );
        } else {
            f.render_widget(Paragraph::new(Text::from("(no results)")).block(block), area);
        }
        return;
    }

    let viewport_height = inner.height.saturating_sub(2) as usize;
    let viewport_width = inner.width as usize;

    if viewport_height == 0 || viewport_width == 0 {
        return;
    }

    let (col_start, col_end) = grid.visible_col_range();
    let vis_cols = &grid.columns[col_start..col_end];
    let vis_widths: Vec<Constraint> =
        grid.column_widths[col_start..col_end].iter().map(|w| Constraint::Length(*w)).collect();

    let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let header_cells: Vec<Cell> =
        vis_cols.iter().map(|c| Cell::from(c.name.as_str()).style(header_style)).collect();
    let header = Row::new(header_cells).height(1);

    let row_start = grid.scroll.row_offset;
    let row_end = (row_start + viewport_height).min(grid.rows.len());
    let visible_rows: Vec<Row> = grid
        .rows
        .iter()
        .skip(row_start)
        .take(row_end - row_start)
        .enumerate()
        .map(|(rel_idx, cells)| {
            let abs_idx = row_start + rel_idx;
            let row_style = if abs_idx == grid.selected_row {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let rendered_cells: Vec<Cell> = cells[col_start..col_end]
                .iter()
                .enumerate()
                .map(|(col_rel, value)| {
                    let abs_col = col_start + col_rel;
                    let is_selected = abs_idx == grid.selected_row && abs_col == grid.selected_col;

                    let display =
                        if value.is_empty() { grid.null_display.clone() } else { value.clone() };

                    if is_selected {
                        Cell::from(display)
                            .style(row_style.bg(Color::DarkGray).add_modifier(Modifier::REVERSED))
                    } else {
                        Cell::from(display).style(row_style)
                    }
                })
                .collect();

            Row::new(rendered_cells)
        })
        .collect();

    let mut table_state = TableState::default();
    let selected_rel = grid.selected_row.saturating_sub(row_start);
    table_state.select(Some(selected_rel));

    let table = Table::new(visible_rows, vis_widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(table, area, &mut table_state);
}

fn render_cell_popup(f: &mut Frame, area: Rect, popup: &crate::state::CellPopupState) {
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

    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("Esc/q/Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ]);

    let content = Paragraph::new(Text::from(lines));

    f.render_widget(content.block(block), popup_area);

    let help_paragraph = Paragraph::new(help).style(Style::default().bg(Color::Black));
    let help_rect =
        Rect { y: popup_area.y + popup_area.height.saturating_sub(3), height: 1, ..popup_area };
    f.render_widget(help_paragraph, help_rect);
}

#[allow(clippy::too_many_arguments)]
fn render_quick_doc(
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

    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("g/G", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":top/bottom  "),
        Span::styled("Esc/q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ]);

    let content = Paragraph::new(Text::from(visible));
    f.render_widget(content.block(block), popup_area);

    let help_paragraph = Paragraph::new(help).style(Style::default().bg(Color::Black));
    let help_rect =
        Rect { y: popup_area.y + popup_area.height.saturating_sub(3), height: 1, ..popup_area };
    f.render_widget(help_paragraph, help_rect);
}

fn render_keymap_help(f: &mut Frame, area: Rect, panel: &Panel, mode: &Mode, scroll: usize) {
    let popup_area = centered_rect(60, 60, area);

    let mode_str = match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Command { .. } => "COMMAND",
        Mode::ConnectDialog { .. } => "CONNECT",
        Mode::Visual => "VISUAL",
    };
    let panel_str = match panel {
        Panel::SchemaExplorer => "Schema Explorer",
        Panel::QueryEditor => "Query Editor",
        Panel::ResultGrid => "Result Grid",
        Panel::Output => "Output",
    };
    let title = format!(" Keymap Help — {} [{}] ", panel_str, mode_str);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    let viewport_height = inner.height.saturating_sub(4) as usize;

    let mut all_lines: Vec<Line> = Vec::new();

    let bindings = keymap_help::get_keybindings(panel, mode);
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

    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":scroll  "),
        Span::styled("Esc/q/?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(":close"),
    ]);

    let content = Paragraph::new(Text::from(visible));
    f.render_widget(content.block(block), popup_area);

    let help_paragraph = Paragraph::new(help).style(Style::default().bg(Color::Black));
    let help_rect =
        Rect { y: popup_area.y + popup_area.height.saturating_sub(3), height: 1, ..popup_area };
    f.render_widget(help_paragraph, help_rect);
}

fn format_count(n: u64) -> String {
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
