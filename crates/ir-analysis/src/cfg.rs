use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminatorKind {
    Branch,
    CondBranch { cond: String, true_label: String, false_label: String },
    Switch { val: String, default_label: String, cases: Vec<(String, String)> },
    Return,
    Unreachable,
    Invoke,
    Resume,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub name: String,
    pub range: Range,
    pub instructions: Vec<String>,
    pub terminator: Option<TerminatorKind>,
    pub successors: Vec<String>,
    pub predecessors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCfg {
    pub function_name: String,
    pub entry_block: String,
    pub blocks: HashMap<String, BasicBlock>,
    pub block_order: Vec<String>,
}
