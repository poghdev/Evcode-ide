mod colors;
mod overlays;
mod status_bar;
mod tabs;

use crate::app::{App, Focus};
use crate::editor::highlighter::highlight_line;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::Paragraph,
    Frame,
};

pub use colors::Colors;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let colors = Colors::from_theme(&app.theme);

    let terminal_height = if app.show_terminal { 12 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // tabs
            Constraint::Min(1),              // editor
            Constraint::Length(terminal_height), // terminal
            Constraint::Length(1),           // statusbar
        ])
        .split(area);

    tabs::draw_tabs(f, chunks[0], app, &colors);

    if let Some(file) = app.editor.files.get_mut(app.editor.current_file) {
        let lang = file.lang;
        let is_ghost = app.ghost_mode;
        let is_focused = app.focus == Focus::Editor;

        let highlighted: Text<'static> = {
            let selection = file.textarea.selection_range().map(|(s, e)| {
                if s < e { (s, e) } else { (e, s) }
            });

            let lines: Vec<ratatui::text::Line<'static>> = file
                .textarea
                .lines()
                .iter()
                .enumerate()
                .map(|(r, l)| {
                    let owned: String = l.clone();
                    let base_line = highlight_line(&owned, lang);
                    let mut spans = Vec::new();
                    
                    let mut current_col = 0;
                    for span in base_line.spans {
                        for c in span.content.chars() {
                            let mut style = span.style;
                            
                            if let Some((start, end)) = &selection {
                                if (r > start.0 || (r == start.0 && current_col >= start.1)) &&
                                   (r < end.0 || (r == end.0 && current_col < end.1)) {
                                    style = style
                                        .bg(colors.selected_bg)
                                        .fg(colors.selected_fg);
                                }
                            }

                            spans.push(ratatui::text::Span::styled(c.to_string(), style));
                            current_col += 1;
                        }
                    }
                    ratatui::text::Line::from(spans)
                })
                .collect();
            Text::from(lines)
        };
        let line_count = file.textarea.lines().len();
        let (cursor_row, _) = file.textarea.cursor();
        let title = file.title();

        let visible_height = chunks[1].height.saturating_sub(2) as usize;

        if cursor_row < file.scroll_top {
            file.scroll_top = cursor_row;
        }
        else if cursor_row >= file.scroll_top + visible_height {
            file.scroll_top = cursor_row - visible_height + 1;
        }

        let scroll_row = file.scroll_top;

        let border_color = if is_focused { colors.accent } else { colors.border };
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(ratatui::text::Span::styled(
                format!(" {title} "),
                Style::default().fg(colors.text),
            ));
        file.textarea.set_block(block);
        file.textarea.set_line_number_style(Style::default().fg(colors.muted));

        let cursor_style = if is_ghost {
            Style::default().fg(colors.ghost)
        } else {
            Style::default().bg(colors.accent).fg(colors.bg)
        };
        file.textarea.set_cursor_style(cursor_style);

        file.textarea.set_style(Style::default().fg(colors.bg));

        f.render_widget(&file.textarea, chunks[1]);

        let line_num_width: u16 = if line_count >= 1000 { 5 } else { 4 };
        let a = chunks[1];
        let inner = Rect {
            x: a.x + 1 + line_num_width,
            y: a.y + 1,
            width: a.width.saturating_sub(2 + line_num_width),
            height: a.height.saturating_sub(2),
        };
        f.render_widget(Paragraph::new(highlighted).scroll((scroll_row as u16, 0)), inner);
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
