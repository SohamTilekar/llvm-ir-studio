use crate::cfg::{BasicBlock, FunctionCfg, TerminatorKind};
use crate::diagnostic::AnalysisDiagnostic;
use crate::symbol::{contains_position, FunctionSymbols, Symbol, SymbolKind, SymbolTable};
use ir_core::{Document, NodeExt};
use ir_version::LlvmVersion;
use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::{DiagnosticSeverity, Range};
use tree_sitter::{Node, Tree};

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub symbols: SymbolTable,
    pub diagnostics: Vec<AnalysisDiagnostic>,
    pub cfgs: HashMap<String, FunctionCfg>,
    pub version: LlvmVersion,
}

pub fn analyze(doc: &Document, tree: &Tree, version: LlvmVersion) -> AnalysisResult {
    let source = doc.text();
    let mut diagnostics = Vec::new();
    let mut symbols = SymbolTable::default();
    let mut cfgs = HashMap::new();

    // 1. Walk Tree-sitter CST for syntax errors and version-specific issues
    let root = tree.root_node();
    walk_ast_diagnostics(root, &source, doc, version, &mut diagnostics);

    // 2. Extract symbols (globals, types, functions, basic blocks, SSA registers)
    extract_symbols(root, &source, doc, &mut symbols);

    // 3. SSA Validation & CFG Extraction
    build_cfgs_and_validate_ssa(doc, &source, &mut symbols, &mut cfgs, &mut diagnostics);

    AnalysisResult {
        symbols,
        diagnostics,
        cfgs,
        version,
    }
}

fn walk_ast_diagnostics(
    node: Node,
    source: &str,
    doc: &Document,
    version: LlvmVersion,
    diags: &mut Vec<AnalysisDiagnostic>,
) {
    if node.is_error() {
        let text = node.text(source);
        let range = node.lsp_range(doc);
        let msg = if text.trim().is_empty() {
            "Syntax error: unexpected or invalid token".to_string()
        } else {
            format!("Syntax error near '{}'", text.trim())
        };
        diags.push(AnalysisDiagnostic::new(
            "LLVM-SYNTAX-001",
            msg,
            range,
            DiagnosticSeverity::ERROR,
        ));
    } else if node.is_missing() {
        let range = node.lsp_range(doc);
        diags.push(AnalysisDiagnostic::new(
            "LLVM-SYNTAX-002",
            format!("Missing expected token '{}'", node.kind()),
            range,
            DiagnosticSeverity::ERROR,
        ));
    }

    // Version-specific checks
    let kind = node.kind();
    let text = node.text(source).trim();

    // Check typed vs opaque pointers
    if kind == "type" || kind == "type_keyword" {
        if text.ends_with('*') {
            if version.requires_opaque_pointers() {
                // LLVM 17+ disallows typed pointers
                let range = node.lsp_range(doc);
                diags.push(
                    AnalysisDiagnostic::new(
                        "LLVM-VER-001",
                        format!(
                            "Typed pointer '{}' is unsupported in {}; use opaque 'ptr'",
                            text, version
                        ),
                        range,
                        DiagnosticSeverity::ERROR,
                    )
                    .with_quick_fix("Convert typed pointer to opaque 'ptr'", range, "ptr"),
                );
            }
        } else if text == "ptr" {
            if !version.supports_opaque_pointers() {
                let range = node.lsp_range(doc);
                diags.push(AnalysisDiagnostic::new(
                    "LLVM-VER-002",
                    format!(
                        "Opaque pointer 'ptr' is not supported in {} (requires LLVM 15+)",
                        version
                    ),
                    range,
                    DiagnosticSeverity::ERROR,
                ));
            }
        }
    }

    // Check deprecated attributes
    if kind == "attribute_name" || kind == "fn_attr" {
        if version.is_attribute_deprecated(text) {
            let range = node.lsp_range(doc);
            let mut diag = AnalysisDiagnostic::new(
                "LLVM-VER-003",
                format!(
                    "Attribute '{}' is deprecated in {}; use 'memory(...)' instead",
                    text, version
                ),
                range,
                DiagnosticSeverity::WARNING,
            );
            if let Some(replacement) = version.suggest_attribute_replacement(text) {
                diag = diag.with_quick_fix(
                    format!("Replace '{}' with '{}'", text, replacement),
                    range,
                    replacement,
                );
            }
            diags.push(diag);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_ast_diagnostics(child, source, doc, version, diags);
    }
}

fn find_child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

fn extract_symbols(root: Node, source: &str, doc: &Document, symbols: &mut SymbolTable) {
    let mut cursor = root.walk();
    for node in root.children(&mut cursor) {
        let kind = node.kind();

        if kind == "global_type" {
            // Type definition e.g. %struct.Point = type { i32, i32 }
            if let Some(var_node) = find_child_of_kind(node, "local_var") {
                let name = var_node.text(source).to_string();
                let sym = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Type,
                    def_range: var_node.lsp_range(doc),
                    type_str: Some(node.text(source).to_string()),
                    detail: Some(format!("type definition {}", name)),
                    references: Vec::new(),
                };
                symbols.types.insert(name, sym);
            }
        } else if kind == "global_global" || kind == "alias" || kind == "ifunc" {
            // Global variable definition e.g. @g = global i32 0
            if let Some(var_node) = find_child_of_kind(node, "global_var") {
                let name = var_node.text(source).to_string();
                let sym = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Global,
                    def_range: var_node.lsp_range(doc),
                    type_str: Some(node.text(source).to_string()),
                    detail: Some(format!("global {}", name)),
                    references: Vec::new(),
                };
                symbols.globals.insert(name, sym);
            }
        } else if kind == "declare" || kind == "fn_define" {
            // Function declaration or definition
            extract_function_symbols(node, source, doc, symbols, kind == "fn_define");
        }
    }

    // After discovering definitions, discover all reference usages
    collect_references(root, source, doc, symbols);
}

fn extract_function_symbols(
    fn_node: Node,
    source: &str,
    doc: &Document,
    symbols: &mut SymbolTable,
    is_definition: bool,
) {
    let header_node = find_child_of_kind(fn_node, "function_header").unwrap_or(fn_node);

    let fn_name_node = header_node
        .child_by_field_name("name")
        .or_else(|| find_child_of_kind(header_node, "global_var"));

    let fn_name = match fn_name_node {
        Some(n) => n.text(source).to_string(),
        None => return,
    };

    let fn_range = fn_node.lsp_range(doc);
    let mut func_symbols = FunctionSymbols {
        name: fn_name.clone(),
        range: fn_range,
        return_type: None,
        args: Vec::new(),
        locals: HashMap::new(),
        basic_blocks: HashMap::new(),
    };

    // Extract return type if present
    if let Some(ty_node) = find_child_of_kind(header_node, "type") {
        func_symbols.return_type = Some(ty_node.text(source).to_string());
    }

    // Extract arguments
    let arg_list_node = header_node
        .child_by_field_name("arguments")
        .or_else(|| find_child_of_kind(header_node, "argument_list"));

    if let Some(arg_list) = arg_list_node {
        let mut arg_cursor = arg_list.walk();
        for child in arg_list.children(&mut arg_cursor) {
            if child.kind() == "argument" {
                if let Some(var_node) = find_child_of_kind(child, "local_var")
                    .or_else(|| find_child_of_kind(child, "value"))
                {
                    let arg_name = var_node.text(source).to_string();
                    let sym = Symbol {
                        name: arg_name.clone(),
                        kind: SymbolKind::Argument,
                        def_range: var_node.lsp_range(doc),
                        type_str: Some(child.text(source).to_string()),
                        detail: Some(format!("parameter {}", arg_name)),
                        references: Vec::new(),
                    };
                    func_symbols.args.push(sym);
                }
            }
        }
    }

    // Record function symbol in globals table
    let fn_sym = Symbol {
        name: fn_name.clone(),
        kind: SymbolKind::Function,
        def_range: fn_name_node.unwrap().lsp_range(doc),
        type_str: func_symbols.return_type.clone(),
        detail: Some(if is_definition {
            format!("function definition {}", fn_name)
        } else {
            format!("function declaration {}", fn_name)
        }),
        references: Vec::new(),
    };
    symbols.globals.insert(fn_name.clone(), fn_sym);

    if !is_definition {
        symbols.functions.insert(fn_name, func_symbols);
        return;
    }

    // Traverse body for basic blocks and local SSA registers
    let body_node = fn_node
        .child_by_field_name("body")
        .or_else(|| find_child_of_kind(fn_node, "function_body"));

    if let Some(body) = body_node {
        let mut body_cursor = body.walk();
        for child in body.children(&mut body_cursor) {
            if child.kind() == "label" {
                let label_name = child.text(source).trim_end_matches(':').trim().to_string();
                let bb_range = child.lsp_range(doc);

                func_symbols.basic_blocks.insert(
                    label_name.clone(),
                    Symbol {
                        name: label_name.clone(),
                        kind: SymbolKind::BasicBlock,
                        def_range: bb_range,
                        type_str: None,
                        detail: Some(format!("basic block {}", label_name)),
                        references: Vec::new(),
                    },
                );
            } else if child.kind() == "instruction" {
                if let Some(first) = child.named_child(0) {
                    if first.kind() == "local_var" {
                        let reg_name = first.text(source).to_string();
                        let sym = Symbol {
                            name: reg_name.clone(),
                            kind: SymbolKind::LocalRegister,
                            def_range: first.lsp_range(doc),
                            type_str: Some(child.text(source).to_string()),
                            detail: Some(format!("SSA register {}", reg_name)),
                            references: Vec::new(),
                        };
                        func_symbols.locals.insert(reg_name, sym);
                    }
                }
            }
        }
    }

    symbols.functions.insert(fn_name, func_symbols);
}

fn collect_references(root: Node, source: &str, doc: &Document, symbols: &mut SymbolTable) {
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        let kind = node.kind();
        let text = node.text(source).trim();

        if kind == "local_var" {
            let range = node.lsp_range(doc);
            // Find enclosing function
            for func in symbols.functions.values_mut() {
                if contains_position(&func.range, range.start) {
                    if let Some(local) = func.locals.get_mut(text) {
                        if local.def_range != range {
                            local.references.push(range);
                        }
                    } else if let Some(arg) = func.args.iter_mut().find(|a| a.name == text) {
                        if arg.def_range != range {
                            arg.references.push(range);
                        }
                    }
                    break;
                }
            }
        } else if kind == "global_var" {
            let range = node.lsp_range(doc);
            if let Some(global) = symbols.globals.get_mut(text) {
                if global.def_range != range {
                    global.references.push(range);
                }
            }
        } else if kind == "label" || kind == "label_name" {
            let range = node.lsp_range(doc);
            let label_name = text.trim_end_matches(':').trim_start_matches('%');
            for func in symbols.functions.values_mut() {
                if contains_position(&func.range, range.start) {
                    if let Some(bb) = func.basic_blocks.get_mut(label_name) {
                        if bb.def_range != range {
                            bb.references.push(range);
                        }
                    }
                    break;
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
}

fn build_cfgs_and_validate_ssa(
    doc: &Document,
    _source: &str,
    symbols: &mut SymbolTable,
    cfgs: &mut HashMap<String, FunctionCfg>,
    diags: &mut Vec<AnalysisDiagnostic>,
) {
    for (fn_name, func) in &symbols.functions {
        let mut cfg = FunctionCfg {
            function_name: fn_name.clone(),
            entry_block: "entry".to_string(),
            blocks: HashMap::new(),
            block_order: Vec::new(),
        };

        // Populate known basic blocks
        for bb_name in func.basic_blocks.keys() {
            cfg.block_order.push(bb_name.clone());
            cfg.blocks.insert(
                bb_name.clone(),
                BasicBlock {
                    name: bb_name.clone(),
                    range: func.basic_blocks[bb_name].def_range,
                    instructions: Vec::new(),
                    terminator: None,
                    successors: Vec::new(),
                    predecessors: Vec::new(),
                },
            );
        }

        // Set entry block as first ordered block if available
        if let Some(first) = cfg.block_order.first() {
            cfg.entry_block = first.clone();
        }

        // Extract instructions and branch targets line by line in function range
        let fn_start_line = func.range.start.line as usize;
        let fn_end_line = func.range.end.line as usize;

        let mut current_block = cfg.entry_block.clone();

        for line_idx in fn_start_line..=fn_end_line {
            if line_idx >= doc.len_lines() {
                break;
            }
            let line_slice = doc.rope().line(line_idx).to_string();
            let trimmed = line_slice.trim();

            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            // Check if this line defines a label e.g. "then:" or "entry:"
            if trimmed.ends_with(':') && !trimmed.contains(' ') {
                let label = trimmed.trim_end_matches(':');
                current_block = label.to_string();
                if !cfg.blocks.contains_key(&current_block) {
                    cfg.blocks.insert(
                        current_block.clone(),
                        BasicBlock {
                            name: current_block.clone(),
                            range: Range {
                                start: tower_lsp::lsp_types::Position {
                                    line: line_idx as u32,
                                    character: 0,
                                },
                                end: tower_lsp::lsp_types::Position {
                                    line: line_idx as u32,
                                    character: trimmed.len() as u32,
                                },
                            },
                            instructions: Vec::new(),
                            terminator: None,
                            successors: Vec::new(),
                            predecessors: Vec::new(),
                        },
                    );
                    cfg.block_order.push(current_block.clone());
                }
                continue;
            }

            if let Some(bb) = cfg.blocks.get_mut(&current_block) {
                bb.instructions.push(trimmed.to_string());

                // Check terminator
                if trimmed.starts_with("ret ") || trimmed == "ret" {
                    bb.terminator = Some(TerminatorKind::Return);
                } else if trimmed.starts_with("br ") {
                    if trimmed.contains("label %") {
                        // Could be unconditional "br label %dest" or conditional "br i1 %cond, label %t, label %f"
                        let labels: Vec<&str> = trimmed
                            .split("label %")
                            .skip(1)
                            .map(|s| s.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').next().unwrap_or(""))
                            .filter(|s| !s.is_empty())
                            .collect();

                        if labels.len() == 1 {
                            bb.terminator = Some(TerminatorKind::Branch);
                            bb.successors.push(labels[0].to_string());
                        } else if labels.len() >= 2 {
                            bb.terminator = Some(TerminatorKind::CondBranch {
                                cond: "%cond".to_string(),
                                true_label: labels[0].to_string(),
                                false_label: labels[1].to_string(),
                            });
                            bb.successors.push(labels[0].to_string());
                            bb.successors.push(labels[1].to_string());
                        }
                    }
                } else if trimmed.starts_with("unreachable") {
                    bb.terminator = Some(TerminatorKind::Unreachable);
                } else if trimmed.starts_with("switch ") {
                    // switch i32 %val, label %default [ i32 0, label %onzero ... ]
                    let labels: Vec<&str> = trimmed
                        .split("label %")
                        .skip(1)
                        .map(|s| s.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').next().unwrap_or(""))
                        .filter(|s| !s.is_empty())
                        .collect();

                    let default_label = labels.first().cloned().unwrap_or("").to_string();
                    let case_labels: Vec<_> = labels.iter().skip(1).map(|&s| ("".to_string(), s.to_string())).collect();
                    for &lbl in &labels {
                        bb.successors.push(lbl.to_string());
                    }
                    bb.terminator = Some(TerminatorKind::Switch {
                        val: "%val".to_string(),
                        default_label,
                        cases: case_labels,
                    });
                }
            }
        }

        // Validate CFG successors & compute predecessors
        let block_names: HashSet<String> = cfg.blocks.keys().cloned().collect();
        let mut preds_map: HashMap<String, Vec<String>> = HashMap::new();

        for (b_name, b_data) in &cfg.blocks {
            for succ in &b_data.successors {
                if !block_names.contains(succ) {
                    diags.push(AnalysisDiagnostic::new(
                        "LLVM-CFG-001",
                        format!("Branch to unknown basic block '%{}'", succ),
                        b_data.range,
                        DiagnosticSeverity::ERROR,
                    ));
                } else {
                    preds_map.entry(succ.clone()).or_default().push(b_name.clone());
                }
            }
        }

        for (b_name, preds) in preds_map {
            if let Some(b) = cfg.blocks.get_mut(&b_name) {
                b.predecessors = preds;
            }
        }

        cfgs.insert(fn_name.clone(), cfg);
    }
}
