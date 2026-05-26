use crate::app::App;
use crate::ui::Colors;
use crate::snap::DiffLine;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn draw_file_tree(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let width = (area.width / 3).max(30).min(50);
    let height = area.height.saturating_sub(4);
    let popup_area = Rect {
        x: area.x,
        y: area.y + 1,
        width,
        height,
    };

    f.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = app
        .file_tree
        .nodes
        .iter()
        .map(|node| {
            let indent = "  ".repeat(node.depth.saturating_sub(1));
            let icon = if node.is_dir { "▸ " } else { "  " };
            let label = format!("{indent}{icon}{}", node.name);
            let style = if node.is_dir {
                Style::default().fg(colors.accent)
            } else {
                Style::default().fg(colors.text)
            };
            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.file_tree.selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.accent))
                .title(Span::styled(
                    " Files (Esc close) ",
                    Style::default().fg(colors.accent),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(colors.selected_bg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, popup_area, &mut state);
}

pub fn draw_teleport(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let width = (area.width * 2 / 3).max(40).min(80);
    let height = 20u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 4;
    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width,
        height,
    };

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .title(Span::styled(
            " Teleport (Ctrl+P) ",
            Style::default().fg(colors.accent),
        ));
    f.render_widget(block, popup_area);

    let input_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width.saturating_sub(2),
        height: 1,
    };
    let input_text = format!("> {}", app.teleport.query);
    let input_para = Paragraph::new(Span::styled(input_text, Style::default().fg(colors.text)));
    f.render_widget(input_para, input_area);

    let sep_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 2,
        width: popup_area.width.saturating_sub(2),
        height: 1,
    };
    let sep = Paragraph::new(Span::styled(
        "─".repeat(sep_area.width as usize),
        Style::default().fg(colors.border),
    ));
    f.render_widget(sep, sep_area);

    let list_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 3,
        width: popup_area.width.saturating_sub(2),
        height: popup_area.height.saturating_sub(4),
    };

    let items: Vec<ListItem> = app
        .teleport
        .results
        .iter()
        .map(|p| {
            let display = if p.starts_with("./") { &p[2..] } else { p };
            let display = if display.len() > (list_area.width as usize).saturating_sub(2) {
                format!("…{}", &display[display.len().saturating_sub(list_area.width as usize - 3)..])
            } else {
                display.to_string()
            };
            ListItem::new(Line::from(Span::styled(
                display,
                Style::default().fg(colors.text),
            )))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.teleport.selected));

    let list = List::new(items)
        .highlight_style(Style::default().bg(colors.selected_bg).fg(colors.accent));

    f.render_stateful_widget(list, list_area, &mut state);
}

pub fn draw_terminal(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(Span::styled(
            " Terminal (Esc close) ",
            Style::default().fg(colors.muted),
        ));
    f.render_widget(block.clone(), area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let history_height = inner.height.saturating_sub(1);
    let history_area = Rect {
        height: history_height,
        ..inner
    };

    let history_lines: Vec<Line> = app
        .term
        .history
        .iter()
        .rev()
        .take(history_height as usize)
        .rev()
        .flat_map(|entry| {
            let cmd_style = Style::default().fg(colors.accent);
            let out_style = if entry.is_error {
                Style::default().fg(colors.error)
            } else {
                Style::default().fg(colors.text)
            };
            vec![
                Line::from(Span::styled(format!("$ {}", entry.command), cmd_style)),
                Line::from(Span::styled(entry.output.trim().to_string(), out_style)),
            ]
        })
        .collect();

    let history_para = Paragraph::new(history_lines);
    f.render_widget(history_para, history_area);

    let input_area = Rect {
        x: inner.x,
        y: inner.y + history_height,
        width: inner.width,
        height: 1,
    };
    let input_text = format!("$ {}_", app.term.input);
    let input_para =
        Paragraph::new(Span::styled(input_text, Style::default().fg(colors.accent)));
    f.render_widget(input_para, input_area);
}

pub fn draw_naming_popup(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let title = match app.naming_type {
        crate::app::NamingType::CreateFile => " New File: ",
        crate::app::NamingType::CreateFolder => " New Folder: ",
        crate::app::NamingType::Rename => " Rename to: ",
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .title(Span::styled(title, Style::default().fg(colors.accent)));
    
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 2 - 1,
        width: area.width / 2,
        height: 3,
    };
    
    f.render_widget(Clear, popup_area);
    let input = Paragraph::new(format!(" {}", app.naming_input)).block(block);
    f.render_widget(input, popup_area);
}

pub fn draw_confirm_delete(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 2 - 2,
        width: area.width / 2,
        height: 5,
    };

    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(vec![Span::raw("Delete "), Span::styled(&app.naming_target, Style::default().fg(colors.error)), Span::raw("?")]),
        Line::from(""),
        Line::from(vec![Span::styled(" [Y] ", Style::default().fg(colors.accent)), Span::raw("Yes  "), Span::styled(" [N] ", Style::default().fg(colors.muted)), Span::raw("No")]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.error))
        .title(" Confirm Deletion ");

    let para = Paragraph::new(text).block(block).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(para, popup_area);
}

pub fn draw_diff_view(f: &mut Frame, area: Rect, app: &App, colors: &Colors) {
    let popup_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(6),
    };

    f.render_widget(Clear, popup_area);

    let mut lines = Vec::new();
    if let Some(diff) = &app.snap_diff {
        for line in diff {
            match line {
                DiffLine::Unchanged(s) => lines.push(Line::from(Span::styled(format!("  {s}"), Style::default().fg(colors.muted)))),
                DiffLine::Added(s) => lines.push(Line::from(Span::styled(format!("+ {s}"), Style::default().fg(ratatui::style::Color::Green)))),
                DiffLine::Removed(s) => lines.push(Line::from(Span::styled(format!("- {s}"), Style::default().fg(colors.error)))),
            }
        }
    } else {
        lines.push(Line::from("Snapshot not found or no changes."));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .title(Span::styled(" Snapshot Comparison (Alt+D - close) ", Style::default().fg(colors.accent)));

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((0, 0));
    
    f.render_widget(para, popup_area);
}
