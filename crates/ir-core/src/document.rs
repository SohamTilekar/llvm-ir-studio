use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent};
use tree_sitter::Point;

#[derive(Clone, Debug)]
pub struct Document {
    rope: Rope,
}

impl Document {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn slice(&self, start_byte: usize, end_byte: usize) -> String {
        let max_len = self.rope.len_bytes();
        let start = start_byte.min(max_len);
        let end = end_byte.min(max_len);
        if start >= end {
            return String::new();
        }
        let char_start = self.rope.byte_to_char(start);
        let char_end = self.rope.byte_to_char(end);
        self.rope.slice(char_start..char_end).to_string()
    }

    pub fn byte_to_point(&self, byte: usize) -> Point {
        let byte = byte.min(self.rope.len_bytes());
        let line = self.rope.byte_to_line(byte);
        let line_start_byte = self.rope.line_to_byte(line);
        let column = byte.saturating_sub(line_start_byte);
        Point { row: line, column }
    }

    pub fn point_to_byte(&self, point: Point) -> usize {
        if point.row >= self.rope.len_lines() {
            return self.rope.len_bytes();
        }
        let line_start_byte = self.rope.line_to_byte(point.row);
        let line_len_bytes = self.rope.line(point.row).len_bytes();
        line_start_byte + point.column.min(line_len_bytes)
    }

    /// Converts a byte offset into an LSP UTF-16 Position
    pub fn byte_to_position(&self, byte: usize) -> Position {
        let byte = byte.min(self.rope.len_bytes());
        let line = self.rope.byte_to_line(byte);
        let line_start_byte = self.rope.line_to_byte(line);
        let _offset_in_line = byte.saturating_sub(line_start_byte);

        let line_slice = self.rope.line(line);
        let char_limit = self.rope.byte_to_char(byte) - self.rope.byte_to_char(line_start_byte);
        let mut utf16_offset = 0;
        for (idx, ch) in line_slice.chars().enumerate() {
            if idx >= char_limit {
                break;
            }
            utf16_offset += ch.len_utf16();
        }

        Position {
            line: line as u32,
            character: utf16_offset as u32,
        }
    }

    /// Converts an LSP UTF-16 Position into a byte offset
    pub fn position_to_byte(&self, pos: Position) -> usize {
        let line_idx = pos.line as usize;
        if line_idx >= self.rope.len_lines() {
            return self.rope.len_bytes();
        }

        let line_start_byte = self.rope.line_to_byte(line_idx);
        let line_slice = self.rope.line(line_idx);

        let mut target_utf16 = pos.character as usize;
        let mut byte_accum = 0;

        for ch in line_slice.chars() {
            let utf16_len = ch.len_utf16();
            if target_utf16 < utf16_len {
                break;
            }
            target_utf16 -= utf16_len;
            byte_accum += ch.len_utf8();
        }

        line_start_byte + byte_accum
    }

    pub fn range_to_byte_range(&self, range: Range) -> std::ops::Range<usize> {
        let start = self.position_to_byte(range.start);
        let end = self.position_to_byte(range.end);
        start..end.max(start)
    }

    pub fn byte_range_to_range(&self, start_byte: usize, end_byte: usize) -> Range {
        Range {
            start: self.byte_to_position(start_byte),
            end: self.byte_to_position(end_byte),
        }
    }

    pub fn apply_changes(&mut self, changes: Vec<TextDocumentContentChangeEvent>) {
        for change in changes {
            if let Some(range) = change.range {
                let start_byte = self.position_to_byte(range.start);
                let end_byte = self.position_to_byte(range.end);
                let char_start = self.rope.byte_to_char(start_byte);
                let char_end = self.rope.byte_to_char(end_byte);
                self.rope.remove(char_start..char_end);
                if !change.text.is_empty() {
                    self.rope.insert(char_start, &change.text);
                }
            } else {
                self.rope = Rope::from_str(&change.text);
            }
        }
    }
}
