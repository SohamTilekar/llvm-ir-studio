use crate::semantic_tokens::{collect_semantic_tokens, get_legend};
use ir_analysis::{
    analyze, contains_position, verify_with_external_tool, AnalysisResult, SymbolKind,
};
use ir_core::{Document, LlvmParser};
use ir_docs::get_instruction_doc;
use ir_version::{detect_version, LlvmIrConfig, LlvmVersion};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tree_sitter::Tree;

pub struct DocumentState {
    pub doc: Document,
    pub tree: Tree,
    pub analysis: AnalysisResult,
    pub version: LlvmVersion,
}

pub struct Backend {
    pub client: Client,
    pub documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
    pub parser: Arc<Mutex<LlvmParser>>,
    pub config: Arc<RwLock<LlvmIrConfig>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            parser: Arc::new(Mutex::new(LlvmParser::new())),
            config: Arc::new(RwLock::new(LlvmIrConfig::default())),
        }
    }

    async fn reanalyze_and_publish(&self, uri: Url, text: String) {
        let file_path = uri.to_file_path().ok();
        let user_ver = self.config.read().await.version;
        let version = detect_version(file_path.as_deref(), &text, user_ver);

        let doc = Document::new(&text);
        let tree = {
            let mut parser = self.parser.lock().await;
            parser.parse(&text, None)
        };

        if let Some(tree) = tree {
            let analysis = analyze(&doc, &tree, version);

            let lsp_diags: Vec<Diagnostic> =
                analysis.diagnostics.iter().map(|d| d.to_lsp()).collect();
            self.client
                .publish_diagnostics(uri.clone(), lsp_diags, None)
                .await;

            let mut docs = self.documents.write().await;
            docs.insert(
                uri,
                DocumentState {
                    doc,
                    tree,
                    analysis,
                    version,
                },
            );
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Load configuration if present in initialization options
        if let Some(opts) = params.initialization_options {
            if let Ok(cfg) = serde_json::from_value::<LlvmIrConfig>(opts) {
                *self.config.write().await = cfg;
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            legend: get_legend(),
                            work_done_progress_options: Default::default(),
                        },
                    ),
                ),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "%".to_string(),
                        "@".to_string(),
                        " ".to_string(),
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: Default::default(),
                    completion_item: None,
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "llvm-ir-ls".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("llvm-ir-ls server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.reanalyze_and_publish(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.reanalyze_and_publish(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let file_path = uri.to_file_path().ok();

        let verify_on_save = self.config.read().await.verify_on_save.unwrap_or(true);
        if verify_on_save {
            if let Some(ref path) = file_path {
                let llvm_as = self.config.read().await.llvm_as_path.clone();
                let ext_diags = verify_with_external_tool(path, llvm_as.as_deref());

                if !ext_diags.is_empty() {
                    let docs = self.documents.read().await;
                    if let Some(state) = docs.get(&uri) {
                        let mut all_diags: Vec<Diagnostic> = state
                            .analysis
                            .diagnostics
                            .iter()
                            .map(|d| d.to_lsp())
                            .collect();
                        for d in ext_diags {
                            all_diags.push(d.to_lsp());
                        }
                        self.client
                            .publish_diagnostics(uri.clone(), all_diags, None)
                            .await;
                    }
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().await;
        docs.remove(&params.text_document.uri);
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let docs = self.documents.read().await;
        let state = match docs.get(&params.text_document.uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        let source = state.doc.text();
        let tokens = collect_semantic_tokens(state.tree.root_node(), &source, &state.doc);
        Ok(Some(SemanticTokensResult::Tokens(tokens)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        if let Some(sym) = state.analysis.symbols.find_symbol_at_position(pos) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                uri,
                sym.def_range,
            ))));
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        if let Some(sym) = state.analysis.symbols.find_symbol_at_position(pos) {
            let mut locs = Vec::new();
            if params.context.include_declaration {
                locs.push(Location::new(uri.clone(), sym.def_range));
            }
            for &ref_range in &sym.references {
                locs.push(Location::new(uri.clone(), ref_range));
            }
            return Ok(Some(locs));
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        // 1. Check if hovering over a symbol
        if let Some(sym) = state.analysis.symbols.find_symbol_at_position(pos) {
            let kind_str = match sym.kind {
                SymbolKind::Function => "Function",
                SymbolKind::Global => "Global Variable",
                SymbolKind::Type => "Type Definition",
                SymbolKind::LocalRegister => "SSA Register",
                SymbolKind::Argument => "Parameter",
                SymbolKind::BasicBlock => "Basic Block Label",
            };

            let detail = sym.detail.as_deref().unwrap_or(&sym.name);
            let type_info = sym
                .type_str
                .as_deref()
                .map(|t| format!("\n```llvm\n{}\n```", t))
                .unwrap_or_default();

            let contents = format!(
                "### {} `{}`\n*{}\n*{}",
                kind_str, sym.name, detail, type_info
            );

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: contents,
                }),
                range: Some(sym.def_range),
            }));
        }

        // 2. Check if hovering over an instruction mnemonic
        let byte_offset = state.doc.position_to_byte(pos);
        let word = extract_word_at_byte(&state.doc.text(), byte_offset);
        if let Some(doc) = get_instruction_doc(&word) {
            let md = doc.to_markdown(state.version);
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: md,
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        // Types
        for (name, sym) in &state.analysis.symbols.types {
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: name.clone(),
                detail: sym.detail.clone(),
                kind: to_lsp_symbol_kind(SymbolKind::Type),
                tags: None,
                range: sym.def_range,
                selection_range: sym.def_range,
                children: None,
                deprecated: None,
            });
        }

        // Globals
        for (name, sym) in &state.analysis.symbols.globals {
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: name.clone(),
                detail: sym.detail.clone(),
                kind: if sym.kind == SymbolKind::Function {
                    tower_lsp::lsp_types::SymbolKind::FUNCTION
                } else {
                    tower_lsp::lsp_types::SymbolKind::VARIABLE
                },
                tags: None,
                range: sym.def_range,
                selection_range: sym.def_range,
                children: None,
                deprecated: None,
            });
        }

        // Functions with basic block children
        for (name, func) in &state.analysis.symbols.functions {
            let mut bb_symbols = Vec::new();
            for (bb_name, bb) in &func.basic_blocks {
                #[allow(deprecated)]
                bb_symbols.push(DocumentSymbol {
                    name: bb_name.clone(),
                    detail: Some("basic block".to_string()),
                    kind: tower_lsp::lsp_types::SymbolKind::STRING,
                    tags: None,
                    range: bb.def_range,
                    selection_range: bb.def_range,
                    children: None,
                    deprecated: None,
                });
            }

            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: name.clone(),
                detail: func.return_type.clone(),
                kind: tower_lsp::lsp_types::SymbolKind::FUNCTION,
                tags: None,
                range: func.range,
                selection_range: func.range,
                children: if bb_symbols.is_empty() {
                    None
                } else {
                    Some(bb_symbols)
                },
                deprecated: None,
            });
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let pos = params.position;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        if let Some(sym) = state.analysis.symbols.find_symbol_at_position(pos) {
            // Local registers and basic block labels are safe to rename within function scope
            if sym.kind == SymbolKind::LocalRegister || sym.kind == SymbolKind::BasicBlock {
                return Ok(Some(PrepareRenameResponse::Range(sym.def_range)));
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = params.new_name;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        if let Some(sym) = state.analysis.symbols.find_symbol_at_position(pos) {
            let mut edits = Vec::new();
            edits.push(TextEdit {
                range: sym.def_range,
                new_text: new_name.clone(),
            });

            for &ref_range in &sym.references {
                edits.push(TextEdit {
                    range: ref_range,
                    new_text: new_name.clone(),
                });
            }

            let mut changes = HashMap::new();
            changes.insert(uri, edits);

            return Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut items = Vec::new();

        // 1. Instruction completions
        let instructions = [
            "ret", "br", "switch", "indirectbr", "invoke", "resume", "unreachable",
            "alloca", "load", "store", "fence", "cmpxchg", "atomicrmw", "getelementptr",
            "add", "sub", "mul", "udiv", "sdiv", "fadd", "fsub", "fmul", "fdiv",
            "urem", "srem", "frem", "shl", "lshr", "ashr", "and", "or", "xor",
            "icmp", "fcmp", "phi", "select", "freeze", "call", "va_arg", "landingpad",
            "trunc", "zext", "sext", "fptrunc", "fpext", "fptoui", "fptosi", "uitofp",
            "sitofp", "ptrtoint", "inttoptr", "bitcast", "addrspacecast",
        ];

        for inst in &instructions {
            items.push(CompletionItem {
                label: inst.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("LLVM IR instruction '{}'", inst)),
                insert_text: Some(inst.to_string()),
                ..CompletionItem::default()
            });
        }

        // 2. Types (respecting version)
        let primitive_types = ["i1", "i8", "i16", "i32", "i64", "i128", "float", "double", "void", "label"];
        for ty in &primitive_types {
            items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some("LLVM primitive type".to_string()),
                ..CompletionItem::default()
            });
        }

        if state.version.supports_opaque_pointers() {
            items.push(CompletionItem {
                label: "ptr".to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some("Opaque pointer (LLVM 15+)".to_string()),
                ..CompletionItem::default()
            });
        }

        // 3. In-scope registers and basic blocks
        for func in state.analysis.symbols.functions.values() {
            if contains_position(&func.range, pos) {
                for arg in &func.args {
                    items.push(CompletionItem {
                        label: arg.name.clone(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: Some("Parameter".to_string()),
                        ..CompletionItem::default()
                    });
                }
                for local in func.locals.values() {
                    items.push(CompletionItem {
                        label: local.name.clone(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: Some("SSA register".to_string()),
                        ..CompletionItem::default()
                    });
                }
                for bb in func.basic_blocks.values() {
                    items.push(CompletionItem {
                        label: format!("%{}", bb.name),
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some("Basic block label".to_string()),
                        ..CompletionItem::default()
                    });
                }
                break;
            }
        }

        // 4. Globals & Functions
        for sym in state.analysis.symbols.globals.values() {
            items.push(CompletionItem {
                label: sym.name.clone(),
                kind: Some(if sym.kind == SymbolKind::Function {
                    CompletionItemKind::FUNCTION
                } else {
                    CompletionItemKind::VARIABLE
                }),
                detail: sym.detail.clone(),
                ..CompletionItem::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.read().await;
        let state = match docs.get(&uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        let mut actions = Vec::new();

        for diag in &state.analysis.diagnostics {
            if ranges_overlap(&diag.range, &params.range) {
                for fix in &diag.quick_fixes {
                    let mut changes = HashMap::new();
                    changes.insert(
                        uri.clone(),
                        vec![TextEdit {
                            range: fix.range,
                            new_text: fix.new_text.clone(),
                        }],
                    );

                    let action = CodeAction {
                        title: fix.title.clone(),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diag.to_lsp()]),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            document_changes: None,
                            change_annotations: None,
                        }),
                        is_preferred: Some(true),
                        disabled: None,
                        command: None,
                        data: None,
                    };
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
        }

        Ok(Some(actions))
    }
}

fn to_lsp_symbol_kind(kind: SymbolKind) -> tower_lsp::lsp_types::SymbolKind {
    match kind {
        SymbolKind::Function => tower_lsp::lsp_types::SymbolKind::FUNCTION,
        SymbolKind::Global => tower_lsp::lsp_types::SymbolKind::VARIABLE,
        SymbolKind::Type => tower_lsp::lsp_types::SymbolKind::STRUCT,
        SymbolKind::LocalRegister => tower_lsp::lsp_types::SymbolKind::VARIABLE,
        SymbolKind::Argument => tower_lsp::lsp_types::SymbolKind::VARIABLE,
        SymbolKind::BasicBlock => tower_lsp::lsp_types::SymbolKind::STRING,
    }
}

fn extract_word_at_byte(text: &str, byte: usize) -> String {
    let bytes = text.as_bytes();
    if byte >= bytes.len() {
        return String::new();
    }

    let mut start = byte;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }

    let mut end = byte;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }

    if start < end {
        text[start..end].to_string()
    } else {
        String::new()
    }
}

fn ranges_overlap(r1: &Range, r2: &Range) -> bool {
    !(r1.end.line < r2.start.line
        || (r1.end.line == r2.start.line && r1.end.character < r2.start.character)
        || r1.start.line > r2.end.line
        || (r1.start.line == r2.end.line && r1.start.character > r2.end.character))
}
