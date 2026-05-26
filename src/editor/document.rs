use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct Document {
    rope: Rope,
}

impl Default for Document {
    fn default() -> Self {
        Self { rope: Rope::new() }
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn line(&self, line: usize) -> Option<RopeSlice<'_>> {
        (line < self.line_count()).then(|| self.rope.line(line))
    }

    pub fn line_slice(&self, line: usize) -> Option<RopeSlice<'_>> {
        if line >= self.rope.len_lines() {
            return None;
        }
        Some(self.rope.line(line))
    }

    pub fn line_text(&self, line: usize) -> Option<String> {
        let mut text = String::new();
        self.line_text_into(line, &mut text).then_some(text)
    }

    pub fn line_text_into(&self, line: usize, out: &mut String) -> bool {
        let Some(slice) = self.line(line) else {
            return false;
        };
        out.clear();
        out.reserve(slice.len_bytes());
        for chunk in slice.chunks() {
            out.push_str(chunk);
        }
        trim_line_ending(out);
        true
    }

    pub fn line_len_chars(&self, line: usize) -> usize {
        self.line_text(line)
            .map(|line| line.graphemes(true).count())
            .unwrap_or(0)
    }

    pub fn line_to_char(&self, line: usize) -> usize {
        let line = line.min(self.line_count().saturating_sub(1));
        self.rope.line_to_char(line)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx.min(self.len_chars()))
    }

    pub fn char_to_position(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.len_chars());
        let line = self.char_to_line(char_idx);
        let line_start = self.line_to_char(line);
        let prefix = self.slice_text(line_start, char_idx);
        let column = prefix.graphemes(true).count();
        (line, column.min(self.line_len_chars(line)))
    }

    pub fn char_at(&self, char_idx: usize) -> Option<char> {
        (char_idx < self.len_chars()).then(|| self.rope.char(char_idx))
    }

    pub fn position_to_char(&self, line: usize, column: usize) -> usize {
        let line_start = self.line_to_char(line);
        let Some(text) = self.line_text(line) else {
            return line_start;
        };
        let char_offset = text
            .graphemes(true)
            .take(column)
            .map(|g| g.chars().count())
            .sum::<usize>();
        line_start + char_offset
    }

    pub fn previous_grapheme_boundary(&self, char_idx: usize) -> usize {
        let char_idx = char_idx.min(self.len_chars());
        if char_idx == 0 {
            return 0;
        }
        let line = self.char_to_line(char_idx);
        let line_start = self.line_to_char(line);
        if char_idx == line_start {
            return char_idx - 1;
        }
        let prefix = self.slice_text(line_start, char_idx);
        let mut offset = 0usize;
        let mut prev = 0usize;
        for grapheme in prefix.graphemes(true) {
            prev = offset;
            offset += grapheme.chars().count();
        }
        line_start + prev
    }

    pub fn next_grapheme_boundary(&self, char_idx: usize) -> usize {
        let char_idx = char_idx.min(self.len_chars());
        if char_idx >= self.len_chars() {
            return self.len_chars();
        }
        let line = self.char_to_line(char_idx);
        let line_start = self.line_to_char(line);
        let line_end = if line + 1 < self.line_count() {
            self.line_to_char(line + 1)
        } else {
            self.len_chars()
        };
        let text = self.slice_text(line_start, line_end);
        let mut offset = 0usize;
        for grapheme in text.graphemes(true) {
            let next = offset + grapheme.chars().count();
            if line_start + offset >= char_idx {
                return (line_start + next).min(self.len_chars());
            }
            offset = next;
        }
        line_end
    }

    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.rope.insert_char(char_idx.min(self.len_chars()), ch);
    }

    pub fn insert_str(&mut self, char_idx: usize, text: &str) {
        if !text.is_empty() {
            self.rope.insert(char_idx.min(self.len_chars()), text);
        }
    }

    pub fn insert_line(&mut self, line: usize, text: &str) {
        let char_idx = if line >= self.line_count() {
            self.len_chars()
        } else {
            self.line_to_char(line)
        };
        self.rope.insert(char_idx, text);
        self.rope.insert_char(char_idx + text.chars().count(), '\n');
    }

    pub fn remove_char_range(&mut self, start: usize, end: usize) {
        let start = start.min(self.len_chars());
        let end = end.min(self.len_chars());
        if start < end {
            self.rope.remove(start..end);
        }
    }

    pub fn delete_line(&mut self, line: usize) {
        if line >= self.line_count() {
            return;
        }

        let start = self.rope.line_to_char(line);
        let end = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1)
        } else {
            self.len_chars()
        };
        self.remove_char_range(start, end);
    }

    pub fn slice_text(&self, start: usize, end: usize) -> String {
        let start = start.min(self.len_chars());
        let end = end.min(self.len_chars());
        if start >= end {
            return String::new();
        }

        let slice = self.rope.slice(start..end);
        let mut text = String::with_capacity(slice.len_bytes());
        for chunk in slice.chunks() {
            text.push_str(chunk);
        }
        text
    }

    pub fn slice(&self, start: usize, end: usize) -> Option<RopeSlice<'_>> {
        let start = start.min(self.len_chars());
        let end = end.min(self.len_chars());
        (start < end).then(|| self.rope.slice(start..end))
    }

    pub fn to_string(&self) -> String {
        let mut text = String::with_capacity(self.rope.len_bytes());
        for chunk in self.rope.chunks() {
            text.push_str(chunk);
        }
        text
    }
}

fn trim_line_ending(text: &mut String) {
    if text.ends_with('\n') {
        text.pop();
    }
    if text.ends_with('\r') {
        text.pop();
    }
}
