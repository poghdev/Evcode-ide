use super::*;
use crate::ui::line_num_width;
use crossterm::event;

impl App {
    pub fn handle_mouse(&mut self, event: event::MouseEvent) {
        use event::MouseEventKind::*;

        if let Down(event::MouseButton::Left) = event.kind {
            let x = event.column;
            let y = event.row;

            if y == 0 {
                let mut current_x = 0;
                for (i, file) in self.editor.files.iter().enumerate() {
                    let tab_width = (4 + file.title().len() + 3) as u16;
                    if x >= current_x && x < current_x + tab_width {
                        self.editor.current_file = i;
                        self.focus = Focus::Editor;
                        return;
                    }
                    current_x += tab_width;
                }
            }

            if self.show_file_tree && y >= 1 {
                let tree_width = 30u16;
                if x < tree_width && y >= 2 {
                    let idx = (y - 2) as usize;
                    if idx < self.file_tree.nodes.len() {
                        self.file_tree.selected = idx;
                        if !self.file_tree.nodes[idx].is_dir {
                            let path = self.file_tree.nodes[idx].path.clone();
                            if self.editor.open_file(&path).is_ok() {
                                self.notify_lsp_did_open(&path);
                            }
                            self.show_file_tree = false;
                            self.focus = Focus::Editor;
                        }
                        return;
                    }
                }
                if x >= tree_width {
                    self.show_file_tree = false;
                    self.focus = Focus::Editor;
                }
            } else if y >= 1 && !self.show_teleport {
                self.focus = Focus::Editor;
                if let Some(file) = self.editor.current_file_mut() {
                    position_editor_cursor_from_mouse(file, x, y);
                }
            }
        }

        if self.focus == Focus::Editor {
            if let Some(file) = self.editor.current_file_mut() {
                match event.kind {
                    event::MouseEventKind::ScrollUp => {
                        scroll_editor_file(file, -3, editor_visible_height(self.show_terminal))
                    }
                    event::MouseEventKind::ScrollDown => {
                        scroll_editor_file(file, 3, editor_visible_height(self.show_terminal))
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn scroll_editor_file(
    file: &mut crate::editor::OpenFile,
    delta: isize,
    visible_height: usize,
) {
    let visible_height = visible_height.max(1);
    let max_scroll = file.buffer.line_count().saturating_sub(visible_height);

    file.scroll_top = if delta.is_negative() {
        file.scroll_top.saturating_sub(delta.unsigned_abs())
    } else {
        file.scroll_top
            .saturating_add(delta as usize)
            .min(max_scroll)
    };

    let (cursor_row, cursor_col) = file.buffer.cursor();
    if cursor_row < file.scroll_top {
        file.buffer.set_cursor(file.scroll_top, cursor_col);
    } else {
        let last_visible = file.scroll_top + visible_height.saturating_sub(1);
        if cursor_row > last_visible {
            file.buffer.set_cursor(last_visible, cursor_col);
        }
    }
}

pub fn position_editor_cursor_from_mouse(
    file: &mut crate::editor::OpenFile,
    x: u16,
    y: u16,
) {
    if y < 2 {
        return;
    }

    let lnw = line_num_width(file.buffer.line_count());
    let text_x = 1 + lnw;
    if x < text_x {
        return;
    }

    let line = file.scroll_top + y.saturating_sub(2) as usize;
    if line >= file.buffer.line_count() {
        return;
    }
    let column = x.saturating_sub(text_x) as usize;
    file.buffer.set_cursor(line, column);
}

pub fn editor_visible_height(show_terminal: bool) -> usize {
    let terminal_height = if show_terminal { 12 } else { 0 };
    crossterm::terminal::size()
        .map(|(_, rows)| rows as usize)
        .unwrap_or(24)
        .saturating_sub(4 + terminal_height)
}