use crate::editor::cursor::Cursor;
use crate::editor::document::Document;
use std::collections::VecDeque;

const MAX_UNDO: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferMove {
    Left,
    Right,
    PreviousWord,
    NextWord,
    Up,
    Down,
    Home,
    End,
    Top,
    Bottom,
}

pub struct EditorBuffer {
    document: Document,
    cursor: Cursor,
    selection_anchor: Option<Cursor>,
    undo_stack: VecDeque<UndoRecord>,
    redo_stack: VecDeque<UndoRecord>,
}

struct UndoRecord {
    start: usize,
    inserted: String,
    deleted: String,
    cursor_before: Cursor,
}

impl Default for EditorBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor: Cursor::default(),
            selection_anchor: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            document: Document::from_str(text),
            cursor: Cursor::default(),
            selection_anchor: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor.line_col()
    }

    pub fn set_cursor(&mut self, line: usize, column: usize) {
        self.cursor = self.cursor.set(line, column, &self.document);
    }

    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    pub fn line_len_chars(&self, line: usize) -> usize {
        self.document.line_len_chars(line)
    }

    pub fn line_text(&self, line: usize) -> Option<String> {
        self.document.line_text(line)
    }

    pub fn line_text_into(&self, line: usize, out: &mut String) -> bool {
        self.document.line_text_into(line, out)
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = self.cursor;
        (anchor != cursor).then(|| {
            let (start, end) = if (anchor.line, anchor.column) <= (cursor.line, cursor.column) {
                (anchor, cursor)
            } else {
                (cursor, anchor)
            };
            (start.line_col(), end.line_col())
        })
    }

    pub fn start_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    pub fn cancel_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn select_all(&mut self) {
        self.move_cursor(BufferMove::Top, false);
        self.selection_anchor = Some(self.cursor);
        self.move_cursor(BufferMove::Bottom, true);
    }

    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        let start_idx = self.document.position_to_char(start.0, start.1);
        let end_idx = self.document.position_to_char(end.0, end.1);
        let text = self.document.slice_text(start_idx, end_idx);
        (!text.is_empty()).then_some(text)
    }

    pub fn text(&self) -> String {
        self.document.to_string()
    }

    pub fn insert_char(&mut self, ch: char) {
        let cursor_before = self.cursor;
        let (start, deleted) = self.take_selection();
        let char_idx = start.unwrap_or_else(|| self.cursor_char_idx());
        if let Some(start) = start {
            self.cursor = self.cursor.set_position_from_char(start, &self.document);
        }

        self.document.insert_char(char_idx, ch);
        self.cursor = self.cursor.set_position_from_char(char_idx + 1, &self.document);

        self.push_undo(UndoRecord {
            start: char_idx,
            inserted: ch.to_string(),
            deleted,
            cursor_before,
        });
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let cursor_before = self.cursor;
        let (start, deleted) = self.take_selection();
        let char_idx = start.unwrap_or_else(|| self.cursor_char_idx());
        if let Some(start) = start {
            self.cursor = self.cursor.set_position_from_char(start, &self.document);
        }

        self.document.insert_str(char_idx, text);
        self.cursor = self
            .cursor
            .set_position_from_char(char_idx + text.chars().count(), &self.document);

        self.push_undo(UndoRecord {
            start: char_idx,
            inserted: text.to_owned(),
            deleted,
            cursor_before,
        });
    }

    pub fn insert_line(&mut self, line: usize, text: &str) {
        let cursor_before = self.cursor;
        let char_idx = if line >= self.line_count() {
            self.document.len_chars()
        } else {
            self.document.line_to_char(line)
        };
        self.document.insert_line(line, text);
        self.cursor.clamp_to_document(&self.document);
        let inserted = format!("{text}\n");
        self.push_undo(UndoRecord {
            start: char_idx,
            inserted,
            deleted: String::new(),
            cursor_before,
        });
    }

    pub fn delete_line(&mut self, line: usize) {
        if line >= self.line_count() {
            return;
        }
        let cursor_before = self.cursor;
        let start = self.document.line_to_char(line);
        let end = if line + 1 < self.line_count() {
            self.document.line_to_char(line + 1)
        } else {
            self.document.len_chars()
        };
        let deleted = self.document.slice_text(start, end);
        self.document.delete_line(line);
        self.cursor.clamp_to_document(&self.document);
        self.selection_anchor = None;

        if !deleted.is_empty() {
            self.push_undo(UndoRecord {
                start,
                inserted: String::new(),
                deleted,
                cursor_before,
            });
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.delete_selection() {
            return;
        }
        let char_idx = self.cursor_char_idx();
        if char_idx >= self.document.len_chars() {
            return;
        }

        let cursor_before = self.cursor;
        let end = self.document.next_grapheme_boundary(char_idx);
        let deleted = self.document.slice_text(char_idx, end);
        self.document.remove_char_range(char_idx, end);
        self.cursor.clamp_to_document(&self.document);
        self.push_undo(UndoRecord {
            start: char_idx,
            inserted: String::new(),
            deleted,
            cursor_before,
        });
    }

    pub fn delete_char_backward(&mut self) {
        if self.delete_selection() {
            return;
        }
        let end = self.cursor_char_idx();
        if end == 0 {
            return;
        }
        let cursor_before = self.cursor;
        let start = self.document.previous_grapheme_boundary(end);
        let deleted = self.document.slice_text(start, end);
        self.document.remove_char_range(start, end);
        let (line, column) = self.document.char_to_position(start);
        self.cursor = self.cursor.set(line, column, &self.document);
        self.push_undo(UndoRecord {
            start,
            inserted: String::new(),
            deleted,
            cursor_before,
        });
    }

    pub fn delete_current_line(&mut self) {
        self.delete_line(self.cursor.line);
        self.cursor.column = 0;
        self.cursor.clamp_to_document(&self.document);
    }

    pub fn move_cursor(&mut self, movement: BufferMove, selecting: bool) {
        if selecting {
            self.start_selection();
        } else {
            self.cancel_selection();
        }

        match movement {
            BufferMove::Left => self.cursor.move_left(&self.document),
            BufferMove::Right => self.cursor.move_right(&self.document),
            BufferMove::PreviousWord => self.move_previous_word(),
            BufferMove::NextWord => self.move_next_word(),
            BufferMove::Up => self.cursor.move_up(&self.document),
            BufferMove::Down => self.cursor.move_down(&self.document),
            BufferMove::Home => self.cursor.move_home(),
            BufferMove::End => self.cursor.move_end(&self.document),
            BufferMove::Top => self.cursor.move_top(),
            BufferMove::Bottom => self.cursor.move_bottom(&self.document),
        }
    }

    pub fn undo(&mut self) -> bool {
        let Some(record) = self.undo_stack.pop_back() else {
            return false;
        };

        let inserted_len = record.inserted.chars().count();
        if inserted_len > 0 {
            self.document
                .remove_char_range(record.start, record.start + inserted_len);
        }
        if !record.deleted.is_empty() {
            self.document.insert_str(record.start, &record.deleted);
        }

        self.cursor = record.cursor_before;
        self.cursor.clamp_to_document(&self.document);
        self.selection_anchor = None;

        self.redo_stack.push_back(record);

        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(record) = self.redo_stack.pop_back() else {
            return false;
        };

        let deleted_len = record.deleted.chars().count();
        if deleted_len > 0 {
            self.document
                .remove_char_range(record.start, record.start + deleted_len);
        }
        if !record.inserted.is_empty() {
            self.document.insert_str(record.start, &record.inserted);
        }

        let cursor_after = record.start + record.inserted.chars().count();
        self.cursor = self
            .cursor
            .set_position_from_char(cursor_after, &self.document);
        self.cursor.clamp_to_document(&self.document);
        self.selection_anchor = None;

        self.undo_stack.push_back(record);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.pop_front();
        }

        true
    }

    fn delete_selection(&mut self) -> bool {
        let cursor_before = self.cursor;
        let (Some(start), deleted) = self.take_selection() else {
            return false;
        };
        self.push_undo(UndoRecord {
            start,
            inserted: String::new(),
            deleted,
            cursor_before,
        });
        true
    }

    fn take_selection(&mut self) -> (Option<usize>, String) {
        let Some((start, end)) = self.selection_range() else {
            return (None, String::new());
        };
        let start_idx = self.document.position_to_char(start.0, start.1);
        let end_idx = self.document.position_to_char(end.0, end.1);
        let deleted = self.document.slice_text(start_idx, end_idx);
        self.document.remove_char_range(start_idx, end_idx);
        self.cursor = self.cursor.set(start.0, start.1, &self.document);
        self.selection_anchor = None;
        (Some(start_idx), deleted)
    }

    fn cursor_char_idx(&self) -> usize {
        self.document
            .position_to_char(self.cursor.line, self.cursor.column)
    }

    fn push_undo(&mut self, record: UndoRecord) {
        if record.inserted.is_empty() && record.deleted.is_empty() {
            return;
        }
        self.redo_stack.clear();
        self.undo_stack.push_back(record);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.pop_front();
        }
    }

    fn move_previous_word(&mut self) {
        let mut idx = self.cursor_char_idx();
        while idx > 0 && self.document.char_at(idx - 1).is_some_and(is_word_separator) {
            idx -= 1;
        }
        while idx > 0 && self.document.char_at(idx - 1).is_some_and(|ch| !is_word_separator(ch)) {
            idx -= 1;
        }
        self.cursor = self.cursor.set_position_from_char(idx, &self.document);
    }

    fn move_next_word(&mut self) {
        let len = self.document.len_chars();
        let mut idx = self.cursor_char_idx();
        while idx < len && self.document.char_at(idx).is_some_and(|ch| !is_word_separator(ch)) {
            idx += 1;
        }
        while idx < len && self.document.char_at(idx).is_some_and(is_word_separator) {
            idx += 1;
        }
        self.cursor = self.cursor.set_position_from_char(idx, &self.document);
    }
}

fn is_word_separator(ch: char) -> bool {
    !(ch.is_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::{BufferMove, EditorBuffer};

    #[test]
    fn utf8_cursor_and_backspace_are_char_safe() {
        let mut buffer = EditorBuffer::from_str("աβ\nrust");
        buffer.move_cursor(BufferMove::Right, false);
        buffer.move_cursor(BufferMove::Right, false);
        buffer.delete_char_backward();

        assert_eq!(buffer.text(), "ա\nrust");
        assert_eq!(buffer.cursor(), (0, 1));
    }

    #[test]
    fn backspace_at_line_start_joins_at_boundary() {
        let mut buffer = EditorBuffer::from_str("abc\ndef");
        buffer.set_cursor(1, 0);
        buffer.delete_char_backward();

        assert_eq!(buffer.text(), "abcdef");
        assert_eq!(buffer.cursor(), (0, 3));
    }

    #[test]
    fn line_insert_and_delete_do_not_copy_document() {
        let mut buffer = EditorBuffer::from_str("one\nthree");
        buffer.insert_line(1, "two");
        assert_eq!(buffer.line_text(1).as_deref(), Some("two"));

        buffer.delete_line(1);
        assert_eq!(buffer.text(), "one\nthree");
    }

    #[test]
    fn selection_extracts_rope_slice() {
        let mut buffer = EditorBuffer::from_str("one\ntwo\nthree");
        buffer.set_cursor(0, 1);
        buffer.start_selection();
        buffer.set_cursor(2, 2);

        assert_eq!(buffer.selected_text().as_deref(), Some("ne\ntwo\nth"));
    }

    #[test]
    fn undo_reverts_insert_delete_and_newline() {
        let mut buffer = EditorBuffer::from_str("ab");
        buffer.set_cursor(0, 1);
        buffer.insert_char('X');
        assert_eq!(buffer.text(), "aXb");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");
        assert_eq!(buffer.cursor(), (0, 1));

        buffer.delete_char_forward();
        assert_eq!(buffer.text(), "a");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");

        buffer.insert_newline();
        assert_eq!(buffer.text(), "a\nb");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");
    }

    #[test]
    fn ctrl_word_movement_uses_word_boundaries() {
        let mut buffer = EditorBuffer::from_str("one two_three  four");
        buffer.move_cursor(BufferMove::NextWord, false);
        assert_eq!(buffer.cursor(), (0, 4));
        buffer.move_cursor(BufferMove::NextWord, false);
        assert_eq!(buffer.cursor(), (0, 15));
        buffer.move_cursor(BufferMove::PreviousWord, false);
        assert_eq!(buffer.cursor(), (0, 4));
    }

    #[test]
    fn redo_reapplies_undone_insert() {
        let mut buffer = EditorBuffer::from_str("ab");
        buffer.set_cursor(0, 1);
        buffer.insert_char('X');
        assert_eq!(buffer.text(), "aXb");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");

        assert!(buffer.redo());
        assert_eq!(buffer.text(), "aXb");

        // Undo again to confirm symmetry.
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");
    }

    #[test]
    fn new_edit_clears_redo_history() {
        let mut buffer = EditorBuffer::from_str("ab");
        buffer.set_cursor(0, 1);
        buffer.insert_char('X');
        buffer.undo();
        assert_eq!(buffer.text(), "ab");

        buffer.insert_char('Y');
        assert_eq!(buffer.text(), "aYb");
        assert!(!buffer.redo(), "redo must be empty after a new edit");
        assert_eq!(buffer.text(), "aYb");
    }

    #[test]
    fn redo_nothing_when_stack_empty() {
        let mut buffer = EditorBuffer::from_str("hello");
        assert!(!buffer.redo());
    }
}