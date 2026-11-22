#[cfg(test)]
mod tests {
    use ir_core::{Document, LlvmParser};
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_document_position_conversions() {
        let text = "define i32 @main() {\n  ret i32 0\n}\n";
        let doc = Document::new(text);

        let pos = Position { line: 1, character: 2 };
        let byte = doc.position_to_byte(pos);
        assert_eq!(byte, 21 + 2); // line 0 is 21 bytes

        let roundtrip_pos = doc.byte_to_position(byte);
        assert_eq!(roundtrip_pos, pos);
    }

    #[test]
    fn test_parser_basic() {
        let mut parser = LlvmParser::new();
        let text = "define i32 @foo(i32 %a) {\n  ret i32 %a\n}\n";
        let tree = parser.parse(text, None).expect("failed to parse");
        let root = tree.root_node();
        assert!(!root.has_error());
    }
}
