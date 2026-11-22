use crate::Document;
use std::ops::Range;
use tower_lsp::lsp_types::Range as LspRange;
use tree_sitter::Node;

pub trait NodeExt {
    fn text<'a>(&self, source: &'a str) -> &'a str;
    fn byte_range(&self) -> Range<usize>;
    fn lsp_range(&self, doc: &Document) -> LspRange;
}

impl<'tree> NodeExt for Node<'tree> {
    fn text<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start_byte();
        let end = self.end_byte();
        if start <= end && end <= source.len() {
            &source[start..end]
        } else {
            ""
        }
    }

    fn byte_range(&self) -> Range<usize> {
        self.start_byte()..self.end_byte()
    }

    fn lsp_range(&self, doc: &Document) -> LspRange {
        doc.byte_range_to_range(self.start_byte(), self.end_byte())
    }
}
