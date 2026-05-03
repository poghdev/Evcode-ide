use crate::app::App;
use crate::ui::Colors;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let mode = match app.focus {
        crate::app::Focus::Editor => {
            if app.ghost_mode {
                "GHOST"
            } else {
                "EDIT"
            }
        }
        crate::app::Focus::FileTree => "FILES",
        crate::app::Focus::Teleport => "SEARCH",
        crate::app::Focus::Terminal => "TERM",
        crate::app::Focus::Naming => "NAME",
        crate::app::Focus::ConfirmDelete => "CONFIRM",
    };

    let cursor_info = if let Some(file) = app.editor.current_file_ref() {
        let (row, col) = file.textarea.cursor();
        format!("{}:{}", row + 1, col + 1)
    } else {
        "0:0".to_string()
    };

    let lsp = app.lsp.status_text();
    let theme = match app.theme {
        crate::app::Theme::Dark => "Dark",
        crate::app::Theme::Light => "Light",
    };

    let left = Line::from(vec![
        Span::styled(
            format!(" {mode} "),
            Style::default().fg(colors.bg).bg(colors.accent),
        ),
        Span::raw(" "),
        Span::styled(app.status_msg.clone(), Style::default().fg(colors.text)),
    ]);

    let right_text = format!("{lsp}  {cursor_info}  {theme}  Ctrl+Q: quit ");
    let right = Line::from(Span::styled(
        right_text,
        Style::default().fg(colors.muted),
    ));

    let para_left = Paragraph::new(left).style(Style::default().bg(colors.bg));
    f.render_widget(para_left, area);

    let right_len = right.width() as u16;
    if area.width > right_len {
        let right_area = Rect {
            x: area.x + area.width - right_len,
            y: area.y,
            width: right_len,
            height: area.height,
        };
        let para_right = Paragraph::new(right).style(Style::default().bg(colors.bg));
        f.render_widget(para_right, right_area);
    }
}
