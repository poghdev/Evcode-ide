mod colors;
mod overlays;
mod status_bar;
mod tabs;

use crate::app::{App, Focus};
use crate::editor::highlighter::highlight_line;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub use colors::Colors;

pub fn line_num_width(line_count: usize) -> u16 {
    if line_count >= 1000 { 5 } else { 4 }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let colors = Colors::from_theme(&app.theme);

    let terminal_height = if app.show_terminal { 12 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(terminal_height),
            Constraint::Length(1),
        ])
        .split(area);

    tabs::draw_tabs(f, chunks[0], app, &colors);

    if let Some(file) = app.editor.files.get_mut(app.editor.current_file) {
        let lang = file.lang;
        let is_focused = app.focus == Focus::Editor;
        let line_count = file.buffer.line_count();
        let (cursor_row, cursor_col) = file.buffer.cursor();
        let title = file.title();
        let is_home = file.path.is_none();
        let visible_height = chunks[1].height.saturating_sub(2) as usize;

        if !is_home {
            let max_scroll = line_count.saturating_sub(visible_height.max(1));
            file.scroll_top = file.scroll_top.min(max_scroll);

            if visible_height > 0 {
                if cursor_row < file.scroll_top {
                    file.scroll_top = cursor_row;
                } else if cursor_row >= file.scroll_top + visible_height {
                    file.scroll_top = cursor_row - visible_height + 1;
                }
            }
        } else {
            file.scroll_top = 0;
        }

        let scroll_row = file.scroll_top;

        let highlighted: Text<'static> = {
            let selection = file.buffer.selection_range().map(|(s, e)| {
                if s < e { (s, e) } else { (e, s) }
            });

            let start = scroll_row;
            let end = (start + visible_height).min(file.buffer.line_count());
            let mut scratch = String::new();
            let mut lines = Vec::with_capacity(end.saturating_sub(start));
            for r in start..end {
                file.buffer.line_text_into(r, &mut scratch);
                let base_line = highlight_line(&scratch, lang);
                let mut spans = Vec::with_capacity(base_line.spans.len());

                let mut current_col = 0;
                for span in base_line.spans {
                    push_styled_runs(
                        &mut spans,
                        span.content.as_ref(),
                        span.style,
                        r,
                        &mut current_col,
                        selection,
                        &colors,
                    );
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            Text::from(lines)
        };

        let border_color = if is_focused { colors.accent } else { colors.border };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(ratatui::text::Span::styled(
                format!(" {title} "),
                Style::default().fg(colors.text),
            ));
        f.render_widget(block, chunks[1]);

        let line_num_width = if is_home {
            0
        } else {
            line_num_width(line_count)
        };
        let a = chunks[1];
        let numbers = if is_home {
            Text::default()
        } else {
            Text::from(
                (scroll_row..(scroll_row + visible_height).min(line_count))
                    .map(|line| {
                        Line::from(Span::styled(
                            format!(
                                "{:>width$} ",
                                line + 1,
                                width = line_num_width as usize - 1
                            ),
                            Style::default().fg(colors.muted),
                        ))
                    })
                    .collect::<Vec<_>>(),
            )
        };
        let number_area = Rect {
            x: a.x + 1,
            y: a.y + 1,
            width: line_num_width,
            height: a.height.saturating_sub(2),
        };


        if !is_home {
            f.render_widget(Paragraph::new(numbers), number_area);
        }
        let inner = Rect {
            x: if is_home {
                a.x + 1
            } else {
                a.x + 1 + line_num_width
            },
            y: a.y + 1,
            width: if is_home {
                a.width.saturating_sub(2)
            } else {
                a.width.saturating_sub(2 + line_num_width)
            },
            height: a.height.saturating_sub(2),
        };
        f.render_widget(Paragraph::new(highlighted), inner);

        if is_focused && cursor_row >= scroll_row && cursor_row < scroll_row + visible_height {
            let x = inner.x + (cursor_col as u16).min(inner.width.saturating_sub(1));
            let y = inner.y + (cursor_row - scroll_row) as u16;
            f.set_cursor_position(ratatui::layout::Position::new(x, y));
        }

    }

    if app.show_terminal {
        overlays::draw_terminal(f, chunks[2], app, &colors);
    }
    
    if app.show_snap_diff {
        overlays::draw_diff_view(f, area, app, &colors);
    }

    status_bar::draw_status_bar(f, chunks[3], app, &colors);

    if app.show_file_tree {
        overlays::draw_file_tree(f, area, app, &colors);
    }
    if app.show_teleport {
        overlays::draw_teleport(f, area, app, &colors);
    }
    if app.focus == Focus::Naming {
        overlays::draw_naming_popup(f, area, app, &colors);
    }
    if app.focus == Focus::ConfirmDelete {
        overlays::draw_confirm_delete(f, area, app, &colors);
    }
}

fn push_styled_runs(
    spans: &mut Vec<Span<'static>>,
    text: &str,
    base_style: Style,
    row: usize,
    current_col: &mut usize,
    selection: Option<((usize, usize), (usize, usize))>,
    colors: &Colors,
) {
    let Some((start, end)) = selection else {
        spans.push(Span::styled(text.to_owned(), base_style));
        *current_col += text.graphemes(true).count();
        return;
    };

    let mut run = String::new();
    let mut run_selected = None;

    for grapheme in text.graphemes(true) {
        let selected = (row > start.0 || (row == start.0 && *current_col >= start.1))
            && (row < end.0 || (row == end.0 && *current_col < end.1));

        if run_selected == Some(selected) || run_selected.is_none() {
            run.push_str(grapheme);
            run_selected = Some(selected);
        } else {
            push_run(spans, std::mem::take(&mut run), base_style, run_selected == Some(true), colors);
            run.push_str(grapheme);
            run_selected = Some(selected);
        }

        *current_col += 1;
    }

    if !run.is_empty() {
        push_run(spans, run, base_style, run_selected == Some(true), colors);
    }
}

fn push_run(
    spans: &mut Vec<Span<'static>>,
    text: String,
    base_style: Style,
    selected: bool,
    colors: &Colors,
) {
    let style = if selected {
        base_style.bg(colors.selected_bg).fg(colors.selected_fg)
    } else {
        base_style
    };
    spans.push(Span::styled(text, style));
}
