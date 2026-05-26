use crate::editor::document::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    preferred_column: usize,
}

impl Cursor {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            preferred_column: column,
        }
    }

    pub fn line_col(self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub fn set(self, line: usize, column: usize, document: &Document) -> Self {
        let mut cursor = Self::new(line, column);
        cursor.clamp_to_document(document);
        cursor.preferred_column = cursor.column;
        cursor
    }

    pub fn set_position_from_char(self, char_idx: usize, document: &Document) -> Self {
        let (line, column) = document.char_to_position(char_idx);
        self.set(line, column, document)
    }

    pub fn clamp_to_document(&mut self, document: &Document) {
        let last_line = document.line_count().saturating_sub(1);
        self.line = self.line.min(last_line);
        self.column = self.column.min(document.line_len_chars(self.line));
        self.preferred_column = self.preferred_column.min(self.column);
    }

    pub fn move_left(&mut self, document: &Document) {
        if self.column > 0 {
            self.column -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.column = document.line_len_chars(self.line);
        }
        self.preferred_column = self.column;
    }

    pub fn move_right(&mut self, document: &Document) {
        let line_len = document.line_len_chars(self.line);
        if self.column < line_len {
            self.column += 1;
        } else if self.line + 1 < document.line_count() {
            self.line += 1;
            self.column = 0;
        }
        self.preferred_column = self.column;
    }

    pub fn move_up(&mut self, document: &Document) {
        if self.line > 0 {
            self.line -= 1;
            self.column = self.preferred_column.min(document.line_len_chars(self.line));
        }
    }

    pub fn move_down(&mut self, document: &Document) {
        if self.line + 1 < document.line_count() {
            self.line += 1;
            self.column = self.preferred_column.min(document.line_len_chars(self.line));
        }
    }

    pub fn move_home(&mut self) {
        self.column = 0;
        self.preferred_column = 0;
    }

    pub fn move_end(&mut self, document: &Document) {
        self.column = document.line_len_chars(self.line);
        self.preferred_column = self.column;
    }

    pub fn move_top(&mut self) {
        self.line = 0;
        self.column = 0;
        self.preferred_column = 0;
    }

    pub fn move_bottom(&mut self, document: &Document) {
        self.line = document.line_count().saturating_sub(1);
        self.column = document.line_len_chars(self.line);
        self.preferred_column = self.column;
    }
}
