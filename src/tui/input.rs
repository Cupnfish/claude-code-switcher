//! Minimal char-based single-line text editor for inline TUI input
//! (new API key, rename). Char-based is sufficient for API keys / names.

/// A simple single-line text editor.
pub struct TextInput {
    content: String,
    /// Byte offset that is always a char boundary.
    cursor: usize,
}

impl TextInput {
    pub fn new(initial: &str) -> Self {
        Self {
            content: initial.to_string(),
            cursor: initial.len(),
        }
    }

    pub fn empty() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    pub fn value(&self) -> &str {
        &self.content
    }

    pub fn cursor_byte(&self) -> usize {
        self.cursor
    }

    pub fn insert(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if let Some((idx, _)) = self.content[..self.cursor].char_indices().last() {
            self.content.replace_range(idx..self.cursor, "");
            self.cursor = idx;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.content.len() {
            let next = self.content[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.content.len());
            self.content.replace_range(self.cursor..next, "");
        }
    }

    pub fn move_left(&mut self) {
        if let Some((idx, _)) = self.content[..self.cursor].char_indices().last() {
            self.cursor = idx;
        }
    }

    pub fn move_right(&mut self) {
        if let Some((_, c)) = self.content[self.cursor..].char_indices().next() {
            self.cursor += c.len_utf8();
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.content.len();
    }
}
