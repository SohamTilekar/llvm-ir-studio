use ir_lsp::Backend;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

#[tokio::test]
async fn test_lsp_full_lifecycle() {
    let (service, _) = LspService::new(|client| Backend::new(client));
    let backend = service.inner();

    // 1. Initialize
    let init_params = InitializeParams {
        process_id: None,
        #[allow(deprecated)]
        root_path: None,
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
    };
    let init_result = backend.initialize(init_params).await.unwrap();
    assert!(init_result.capabilities.hover_provider.is_some());
    assert!(init_result.capabilities.definition_provider.is_some());
    assert!(init_result.capabilities.references_provider.is_some());
    assert!(init_result.capabilities.completion_provider.is_some());
    assert!(init_result.capabilities.code_action_provider.is_some());
    assert!(init_result.capabilities.document_symbol_provider.is_some());
    assert!(init_result.capabilities.rename_provider.is_some());

    // 2. Open a document
    let uri = Url::parse("file:///workspace/test.ll").unwrap();
    let source_code = r#"
define i32 @compute(ptr %data, i32 %n) {
entry:
  %val = load i32, ptr %data, align 4
  %res = add i32 %val, %n
  ret i32 %res
}
"#
    .to_string();

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "llvm".to_string(),
                version: 1,
                text: source_code.clone(),
            },
        })
        .await;

    // 3. Test Semantic Tokens
    let sem_tokens = backend
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(sem_tokens.is_some());
    if let Some(SemanticTokensResult::Tokens(tokens)) = sem_tokens {
        assert!(!tokens.data.is_empty(), "Expected semantic tokens to be emitted");
    }

    // 4. Test Hover on instruction mnemonic 'load'
    // Line 3: "  %val = load i32, ptr %data, align 4" -> 'load' starts at char 9
    let hover_inst = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 3, character: 10 },
            },
            work_done_progress_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(hover_inst.is_some(), "Hover on 'load' instruction should return doc");
    if let Some(hover) = hover_inst {
        if let HoverContents::Markup(m) = hover.contents {
            assert!(m.value.contains("load"), "Hover content should explain 'load'");
            assert!(m.value.contains("Memory Access"));
        }
    }

    // 5. Test Hover on symbol '%val'
    let hover_sym = backend
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 3, character: 3 },
            },
            work_done_progress_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(hover_sym.is_some(), "Hover on '%val' register should return definition info");

    // 6. Test Go-To-Definition on reference '%val' in line 4: "%res = add i32 %val, %n"
    let def_res = backend
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 4, character: 18 }, // on %val
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(def_res.is_some(), "Go to definition for %val should succeed");

    // 7. Test Find References on '%val'
    let refs = backend
        .references(ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 3, character: 3 },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        })
        .await
        .unwrap();

    assert!(refs.is_some());
    let refs_vec = refs.unwrap();
    assert!(!refs_vec.is_empty(), "Should find references for %val");

    // 8. Test Document Symbols
    let symbols_res = backend
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(symbols_res.is_some());
    if let Some(DocumentSymbolResponse::Nested(syms)) = symbols_res {
        assert!(syms.iter().any(|s| s.name == "@compute"));
    }

    // 9. Test Completion
    let completions = backend
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 4, character: 2 },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(completions.is_some());
    if let Some(CompletionResponse::Array(items)) = completions {
        assert!(items.iter().any(|i| i.label == "getelementptr"));
        assert!(items.iter().any(|i| i.label == "ptr"));
        assert!(items.iter().any(|i| i.label == "i32"));
    }

    // 9b. Test completion when user types '@' (sigil prefix replacement without doubling)
    let at_uri = Url::parse("file:///workspace/at_test.ll").unwrap();
    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: at_uri.clone(),
                language_id: "llvm".to_string(),
                version: 1,
                text: "define void @calculate() {\nentry:\n  call void @calc\n  ret void\n}\n".to_string(),
            },
        })
        .await;

    // Position is line 2, character 17 (right after '@calc')
    let at_completions = backend
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: at_uri.clone() },
                position: Position { line: 2, character: 17 },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(at_completions.is_some());
    if let Some(CompletionResponse::Array(items)) = at_completions {
        let calc_item = items.iter().find(|i| i.label == "@calculate");
        assert!(calc_item.is_some(), "Should find @calculate in completions");
        let calc_item = calc_item.unwrap();
        // Verify text_edit range replaces from character 12 ('@') to 17 (end of '@calc')
        if let Some(CompletionTextEdit::Edit(edit)) = &calc_item.text_edit {
            assert_eq!(edit.new_text, "@calculate");
            assert_eq!(edit.range.start, Position { line: 2, character: 12 });
            assert_eq!(edit.range.end, Position { line: 2, character: 17 });
        } else {
            panic!("Expected CompletionTextEdit::Edit on @calculate");
        }
    }

    // 9c. Test completion when user types '%' (e.g. '%v')
    let pct_uri = Url::parse("file:///workspace/pct_test.ll").unwrap();
    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: pct_uri.clone(),
                language_id: "llvm".to_string(),
                version: 1,
                text: "define i32 @foo(i32 %val) {\nentry:\n  %r = add i32 %v, 1\n  ret i32 %r\n}\n".to_string(),
            },
        })
        .await;

    // Position is line 2, character 17 (right after '%v')
    let pct_completions = backend
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: pct_uri.clone() },
                position: Position { line: 2, character: 17 },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        })
        .await
        .unwrap();

    assert!(pct_completions.is_some());
    if let Some(CompletionResponse::Array(items)) = pct_completions {
        let val_item = items.iter().find(|i| i.label == "%val");
        assert!(val_item.is_some(), "Should find %val in completions");
        let val_item = val_item.unwrap();
        // Verify text_edit range replaces from character 15 ('%') to 17 (end of '%v')
        if let Some(CompletionTextEdit::Edit(edit)) = &val_item.text_edit {
            assert_eq!(edit.new_text, "%val");
            assert_eq!(edit.range.start, Position { line: 2, character: 15 });
            assert_eq!(edit.range.end, Position { line: 2, character: 17 });
        } else {
            panic!("Expected CompletionTextEdit::Edit on %val");
        }
    }

    // 10. Test Prepare Rename & Rename on '%res'
    let prep_rename = backend
        .prepare_rename(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line: 4, character: 3 }, // %res
        })
        .await
        .unwrap();

    assert!(prep_rename.is_some());

    let rename_result = backend
        .rename(RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 4, character: 3 },
            },
            new_name: "%output".to_string(),
            work_done_progress_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(rename_result.is_some());
    let edit = rename_result.unwrap();
    assert!(edit.changes.is_some());

    // 11. Test Code Action for Version Migration (typed pointer -> ptr in LLVM 18)
    backend.config.write().await.version = Some(18);

    let legacy_uri = Url::parse("file:///workspace/legacy.ll").unwrap();
    let legacy_code = "define void @foo(i32* %p) {\n  ret void\n}\n".to_string();

    backend
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: legacy_uri.clone(),
                language_id: "llvm".to_string(),
                version: 1,
                text: legacy_code,
            },
        })
        .await;

    let actions = backend
        .code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: legacy_uri.clone() },
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 30 },
            },
            context: CodeActionContext::default(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .unwrap();

    assert!(actions.is_some());
    let actions_vec = actions.unwrap();
    assert!(
        actions_vec.iter().any(|a| match a {
            CodeActionOrCommand::CodeAction(ca) => ca.title.contains("Convert typed pointer"),
            _ => false,
        }),
        "Should offer quick fix to convert typed pointer to opaque 'ptr'"
    );
}
