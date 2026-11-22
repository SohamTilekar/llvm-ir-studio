#[cfg(test)]
mod tests {
    use ir_analysis::*;
    use ir_core::{Document, LlvmParser, NodeExt};
    use ir_version::LlvmVersion;

    #[test]
    fn test_valid_ir_analysis() {
        let text = r#"
define i32 @add(i32 %a, i32 %b) {
entry:
  %sum = add i32 %a, %b
  ret i32 %sum
}
"#;
        let mut parser = LlvmParser::new();
        let doc = Document::new(text);
        let tree = parser.parse(text, None).unwrap();
        let mut cursor = tree.root_node().walk();
        for child in tree.root_node().children(&mut cursor) {
            println!("CHILD KIND: {} ({})", child.kind(), child.text(text));
        }
        let res = analyze(&doc, &tree, LlvmVersion::V18);

        assert!(res.diagnostics.is_empty(), "Expected no diagnostics, got {:?}", res.diagnostics);
        assert!(res.symbols.globals.contains_key("@add"));
        assert!(res.cfgs.contains_key("@add"));

        let cfg = &res.cfgs["@add"];
        assert!(cfg.blocks.contains_key("entry"));
    }

    #[test]
    fn test_syntax_error_detection() {
        let text = r#"
define i32 @broken( {
  ret i32 0
"#;
        let mut parser = LlvmParser::new();
        let doc = Document::new(text);
        let tree = parser.parse(text, None).unwrap();
        let res = analyze(&doc, &tree, LlvmVersion::V18);

        assert!(!res.diagnostics.is_empty());
        assert!(res.diagnostics.iter().any(|d| d.code.starts_with("LLVM-SYNTAX")));
    }

    #[test]
    fn test_version_error_detection_typed_pointers() {
        let text = r#"
define void @legacy(i32* %ptr) {
entry:
  ret void
}
"#;
        let mut parser = LlvmParser::new();
        let doc = Document::new(text);
        let tree = parser.parse(text, None).unwrap();

        // In LLVM 18, typed pointer is rejected
        let res18 = analyze(&doc, &tree, LlvmVersion::V18);
        assert!(res18.diagnostics.iter().any(|d| d.code == "LLVM-VER-001"));
        assert!(!res18.diagnostics[0].quick_fixes.is_empty());

        // In LLVM 14, typed pointer is allowed
        let res14 = analyze(&doc, &tree, LlvmVersion::V14);
        assert!(!res14.diagnostics.iter().any(|d| d.code == "LLVM-VER-001"));
    }

    #[test]
    fn test_cfg_branching() {
        let text = r#"
define i32 @max(i32 %a, i32 %b) {
entry:
  %cmp = icmp sgt i32 %a, %b
  br i1 %cmp, label %then, label %else

then:
  ret i32 %a

else:
  ret i32 %b
}
"#;
        let mut parser = LlvmParser::new();
        let doc = Document::new(text);
        let tree = parser.parse(text, None).unwrap();
        let res = analyze(&doc, &tree, LlvmVersion::V18);

        let cfg = res.cfgs.get("@max").expect("cfg for @max");
        assert_eq!(cfg.blocks.len(), 3);
        assert_eq!(cfg.blocks["entry"].successors, vec!["then", "else"]);
        assert_eq!(cfg.blocks["then"].predecessors, vec!["entry"]);
        assert_eq!(cfg.blocks["else"].predecessors, vec!["entry"]);
    }
}
