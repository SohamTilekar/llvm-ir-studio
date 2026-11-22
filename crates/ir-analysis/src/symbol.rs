use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Global,
    Type,
    LocalRegister,
    Argument,
    BasicBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub def_range: Range,
    pub type_str: Option<String>,
    pub detail: Option<String>,
    pub references: Vec<Range>,
}

impl Symbol {
    pub fn new(name: impl Into<String>, kind: SymbolKind, def_range: Range) -> Self {
        Self {
            name: name.into(),
            kind,
            def_range,
            type_str: None,
            detail: None,
            references: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionSymbols {
    pub name: String,
    pub range: Range,
    pub return_type: Option<String>,
    pub args: Vec<Symbol>,
    pub locals: HashMap<String, Symbol>,
    pub basic_blocks: HashMap<String, Symbol>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolTable {
    pub globals: HashMap<String, Symbol>,
    pub types: HashMap<String, Symbol>,
    pub functions: HashMap<String, FunctionSymbols>,
}

impl SymbolTable {
    pub fn find_symbol_at_position(&self, pos: tower_lsp::lsp_types::Position) -> Option<&Symbol> {
        // First check in functions
        for func in self.functions.values() {
            if contains_position(&func.range, pos) {
                // Check locals
                for local in func.locals.values() {
                    if contains_position(&local.def_range, pos)
                        || local.references.iter().any(|r| contains_position(r, pos))
                    {
                        return Some(local);
                    }
                }
                // Check basic blocks
                for bb in func.basic_blocks.values() {
                    if contains_position(&bb.def_range, pos)
                        || bb.references.iter().any(|r| contains_position(r, pos))
                    {
                        return Some(bb);
                    }
                }
                // Check args
                for arg in &func.args {
                    if contains_position(&arg.def_range, pos)
                        || arg.references.iter().any(|r| contains_position(r, pos))
                    {
                        return Some(arg);
                    }
                }
            }
        }

        // Check globals & types
        for sym in self.globals.values().chain(self.types.values()) {
            if contains_position(&sym.def_range, pos)
                || sym.references.iter().any(|r| contains_position(r, pos))
            {
                return Some(sym);
            }
        }

        None
    }
}

pub fn contains_position(range: &Range, pos: tower_lsp::lsp_types::Position) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}
