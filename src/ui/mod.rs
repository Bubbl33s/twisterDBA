pub mod dialog;
pub mod editor;
pub mod explorer;
pub mod output;
pub mod popup;
pub mod result;
pub mod status;
pub mod utils;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::Paragraph,
};

use crate::state::{AppState, Mode, PopupState, SplitDirection};
use crate::theme::Theme;

pub fn render(f: &mut Frame, state: &AppState) {
    let full_area = f.area();
    let theme = &state.theme;

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(full_area);

    render_main_area(f, main_layout[0], state, theme);
    status::render_status_bar(f, main_layout[1], state, theme);

    if let Mode::ConnectDialog { form } = &state.mode {
        utils::render_dialog_backdrop(f, full_area);
        dialog::render_connect_dialog(f, full_area, form, &state.config.connections);
    }

    if let Some(ref popup) = state.cell_popup {
        popup::render_cell_popup(f, full_area, popup);
    }

    match &state.popup {
        PopupState::QuickDoc { schema, table, ddl, row_count, table_size, loading, scroll } => {
            popup::render_quick_doc(
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
            popup::render_keymap_help(f, full_area, &state.focused_window, &state.mode, *scroll);
        },
        PopupState::None => {},
    }
}

fn render_main_area(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(2),
            Constraint::Percentage(80),
        ])
        .split(area);

    explorer::render_database_explorer(f, chunks[0], state, &state.explorer, theme);

    let right_area = chunks[2];
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(right_area);

    let editor_area = right_chunks[0];
    let output_area = right_chunks[1];

    let editor_count = state.editor_splits.len();
    if editor_count == 1 {
        editor::render_single_editor(f, editor_area, state, 0, theme);
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
                    editor::render_single_editor(f, *ec, state, i, theme);
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
                    editor::render_single_editor(f, *ec, state, i, theme);
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

    output::render_output_panel(f, output_area, state, theme);
}
