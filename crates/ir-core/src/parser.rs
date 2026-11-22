use tree_sitter::{Parser, Tree};

pub struct LlvmParser {
    parser: Parser,
}

impl Default for LlvmParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LlvmParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_llvm::LANGUAGE.into())
            .expect("Failed to load LLVM tree-sitter grammar");
        Self { parser }
    }

    pub fn parse(&mut self, text: &str, old_tree: Option<&Tree>) -> Option<Tree> {
        self.parser.parse(text, old_tree)
    }
}
