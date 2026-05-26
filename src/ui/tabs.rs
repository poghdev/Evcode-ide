use crate::app::App;
use crate::ui::Colors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Tabs},
    Frame,
};

pub fn draw_tabs(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let titles: Vec<Line> = app
        .editor
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let num = format!("[{}] ", i + 1);
            let name = file.title();
            Line::from(vec![
                Span::styled(num, Style::default().fg(colors.muted)),
                Span::styled(name, Style::default().fg(colors.text)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default())
        .select(app.editor.current_file)
        .style(Style::default().fg(colors.muted).bg(colors.bg))
        .highlight_style(
            Style::default()
                .fg(colors.accent)
                .bg(colors.bg)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" │ ", Style::default().fg(colors.border)));

    f.render_widget(tabs, area);
}
