use ir_core::{Document, NodeExt};
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
};
use tree_sitter::Node;

pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,  // 0
    SemanticTokenType::VARIABLE,  // 1
    SemanticTokenType::PARAMETER, // 2
    SemanticTokenType::TYPE,      // 3
    SemanticTokenType::KEYWORD,   // 4
    SemanticTokenType::OPERATOR,  // 5
    SemanticTokenType::COMMENT,   // 6
    SemanticTokenType::STRING,    // 7
    SemanticTokenType::NUMBER,    // 8
    SemanticTokenType::new("label"), // 9
    SemanticTokenType::PROPERTY,  // 10
];

pub fn get_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RawToken {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: u32,
}

pub fn collect_semantic_tokens(root: Node, source: &str, doc: &Document) -> SemanticTokens {
    let mut raw_tokens = Vec::new();
    walk_and_collect(root, source, doc, &mut raw_tokens);

    // Sort tokens by line, then by start character
    raw_tokens.sort_by(|a, b| {
        if a.line != b.line {
            a.line.cmp(&b.line)
        } else {
            a.start.cmp(&b.start)
        }
    });

    // Remove duplicates or overlapping tokens
    raw_tokens.dedup_by(|a, b| a.line == b.line && a.start == b.start);

    // Delta encode
    let mut data = Vec::with_capacity(raw_tokens.len());
    let mut prev_line = 0;
    let mut prev_start = 0;

    for tok in raw_tokens {
        let delta_line = tok.line.saturating_sub(prev_line);
        let delta_start = if delta_line == 0 {
            tok.start.saturating_sub(prev_start)
        } else {
            tok.start
        };

        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: tok.length,
            token_type: tok.token_type,
            token_modifiers_bitset: 0,
        });

        prev_line = tok.line;
        prev_start = tok.start;
    }

    SemanticTokens {
        result_id: None,
        data,
    }
}

fn walk_and_collect(node: Node, source: &str, doc: &Document, tokens: &mut Vec<RawToken>) {
    let kind = node.kind();
    let token_type_idx = match kind {
        "function_header" | "function_decl" => None,
        "global_var" => Some(0), // function / global
        "argument" => Some(2),   // parameter
        "local_var" => Some(1),  // variable
        "type" | "type_keyword" => Some(3), // type
        "comment" => Some(6),    // comment
        "string" | "cstring" => Some(7), // string
        "number" | "float" => Some(8),   // number
        "label" | "label_name" => Some(9), // label
        "metadata" | "metadata_node" => Some(10), // property / metadata
        "inst_name" => Some(5),  // operator / instruction
        "define" | "declare" | "target" | "triple" | "datalayout" | "global" | "constant"
        | "align" | "section" | "comdat" | "to" | "inbounds" => Some(4), // keyword
        _ => {
            // Also check if text matches common keywords or instructions
            if node.child_count() == 0 {
                let text = node.text(source).trim();
                if ir_docs::get_instruction_doc(text).is_some() {
                    Some(5)
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    if let Some(type_idx) = token_type_idx {
        let range = node.lsp_range(doc);
        if range.start.line == range.end.line && range.end.character > range.start.character {
            tokens.push(RawToken {
                line: range.start.line,
                start: range.start.character,
                length: range.end.character - range.start.character,
                token_type: type_idx,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_and_collect(child, source, doc, tokens);
    }
}
