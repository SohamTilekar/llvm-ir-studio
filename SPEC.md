Here is the revised, leaner blueprint—**no MCP server**, **LLVM IR version awareness** baked in from Phase 1, and strict incremental phases so Antigravity builds one solid layer at a time.

---

# Project Blueprint: `llvm-ir-ls` (LLVM IR Language Server & Visualizer)

## 1. Core Vision

Build a **single, lightweight native Rust binary** (`llvm-ir-ls`) that acts as a universal LSP server for LLVM IR across VS Code, Zed, Neovim, and Helix.

**Core Principles:**

* **Zero `libLLVM` C++ linking.** Fast builds, tiny memory footprint, no version lock-in.
* **Version-Aware by Design.** LLVM IR syntax evolves across versions (e.g., typed pointers `i32*` pre-LLVM 15 vs. opaque pointers `ptr` in LLVM 15+, attribute/intrinsic changes). The server must understand which LLVM version it is targeting.
* **Incremental & Error-Tolerant.** Uses `tree-sitter` so live editing never crashes the parser.
* **AI-Ready via LSP (No MCP Server).** AI coding assistants inside editors (Antigravity, VS Code, Zed, etc.) already read LSP diagnostics, hover info, and code actions. Make diagnostics rich, machine-readable, and actionable so AI agents can self-correct IR directly from LSP output.

---

## 2. Technology Stack

| Layer | Technology | Role |
| --- | --- | --- |
| **Language** | **Rust** (Cargo workspace) | Single fast native binary, low RAM usage, native fit for Zed WASM extensions. |
| **Parser** | **`tree-sitter` + `tree-sitter-llvm**` | Incremental CST with error recovery on half-typed `.ll` files. |
| **LSP Server** | **`tower-lsp`** + **`tokio`** | Async JSON-RPC LSP over `stdio`. |
| **Text Handling** | **`ropey`** | Fast incremental text buffer + exact UTF-8 ↔ UTF-16 position mapping. |
| **CLI / Config** | **`clap`**, **`serde`**, **`serde_json`**, **`toml`** | Subcommands + `.llvm-ir-ls.toml` version config support. |
| **Optional Verifier** | **`opt` / `llvm-as**` (auto-detected on `PATH`) | Optional on-save deep verification + automatic local LLVM version detection. |

---

## 3. Clean Workspace Structure

```text
llvm-ir-ls/
├── Cargo.toml
├── crates/
│   ├── ir-core/          # Tree-sitter parser, ropey document store, UTF-16 mapper
│   ├── ir-version/       # LLVM IR version profiles (14–21+), version detector, compatibility rules
│   ├── ir-analysis/      # Syntax/semantic diagnostics, symbol table, SSA def-use, CFG
│   ├── ir-docs/          # Version-tagged LLVM LangRef hover docs
│   ├── ir-viz/           # CFG / SSA exporter (Mermaid, DOT, HTML/SVG)
│   ├── ir-lsp/           # tower-lsp handlers (diagnostics, semantic tokens, nav, edit, AI-friendly hints)
│   └── llvm-ir-ls/       # Main CLI binary (`lsp`, `check`, `cfg`)
├── editors/
│   ├── vscode/           # Thin TS LSP client + TextMate grammar + CFG Webview (Phase 3)
│   ├── zed/              # Thin Rust/WASM Zed extension
│   └── configs/          # Neovim & Helix configs
└── tests/
    └── fixtures/         # Versioned `.ll` test files (valid, syntax-broken, opaque vs typed ptr)

```

*(Note: Never log to `stdout` in LSP mode—route all `tracing` logs to `stderr`.)*

---

## 4. Phased Implementation Plan (Execute Strictly in Order)

### Phase 1: Syntax Error Detection, LLVM Version Knowledge & Syntax Highlighting

*Focus: Get a working LSP in editors that highlights IR cleanly, knows the LLVM version, and catches syntax/version errors immediately.*

1. **Workspace & `ir-core**`:
* Scaffold `ir-core`, `ir-version`, `ir-analysis`, `ir-lsp`, and `llvm-ir-ls`.
* Parse `.ll` buffers incrementally using `tree-sitter` + `tree-sitter-llvm` and map byte offsets to LSP UTF-16 `(line, character)`.


2. **LLVM IR Version Knowledge (`ir-version`)**:
* Define version profiles (e.g., LLVM 14, 15, 16, 17, 18, 19, 20, 21) tracking key IR shifts:
* **Opaque pointers (`ptr`)**: Allowed in 15+, mandatory in 17+ (flag `i32*` typed pointers as deprecated/invalid based on target version).
* **Memory/instruction syntax changes**: e.g., `getelementptr` explicit type syntax, `load`/`store` opaque pointer forms, deprecated attributes (`readnone`/`readonly` → `memory(...)` in LLVM 16+).


* **Version Resolution Order**:
1. Explicit `.llvm-ir-ls.toml` or LSP initialization setting (`llvmVersion = 18`).
2. Auto-detect installed `llvm-as --version` / `opt --version` on `PATH`.
3. Heuristic fallback from file content (e.g., presence of `ptr` vs typed pointers) defaulting to latest stable.




3. **Syntax Error Detection**:
* Walk Tree-sitter `ERROR` and `MISSING` nodes and emit precise LSP diagnostics with clear messages.
* Emit version-compatibility warnings/errors (e.g., *"Typed pointer `i32*` is unsupported in LLVM 18; use opaque `ptr`"*).


4. **Syntax Highlighting**:
* Implement LSP **Semantic Tokens** (`textDocument/semanticTokens/full`) covering: functions (`@foo`), globals, local SSA registers (`%0`, `%tmp`), basic block labels, types (`i32`, `ptr`), instructions/keywords, attributes, metadata (`!dbg`), comments, and constants.
* Add minimal **VS Code** (TextMate grammar + LSP launch) and **Zed** (`extension.toml` + WASM hook) wrappers so highlighting + diagnostics work in both editors.



* **Phase 1 Gate:** `cargo test` passes on valid, syntax-broken, and multi-version `.ll` fixtures; opening a `.ll` file in VS Code / Zed shows highlighting and live syntax/version diagnostics.

---

### Phase 2: Navigation, Editing & Comprehension

*Focus: Make reading and writing `.ll` files fast and ergonomic.*

1. **Navigation**:
* **Go-to-Definition**: `%local` → defining instruction, `br label %bb` → block label, `@fn`/`@glob` → `define`/`declare`/global def, `%Struct` → type def.
* **Find All References**: Full SSA def-use chain for `%registers` and all call sites for `@functions`.


2. **Comprehension**:
* **Hover Docs (`ir-docs`)**: Hovering an instruction (`getelementptr`, `phi`, `icmp`, `load`, `store`, `alloca`, `call`, `br`, `switch`, `select`, `bitcast`) shows syntax, behavior, and version notes; hovering `%var` or `@fn` shows its type and definition line.
* **Document Symbols**: Outline of Module → Globals, Struct Types, Functions → Basic Blocks.


3. **Editing**:
* **Rename Symbol**: Safely rename `%local` registers and basic block labels within their enclosing function scope.
* **Autocompletion**: Instruction mnemonics, types (respecting active LLVM version—e.g., suggesting `ptr` in 15+), and in-scope `%registers`, `@globals`, and labels.
* **Optional On-Save Verifier**: If `opt` / `llvm-as` is on `PATH`, run `opt -passes=verify -disable-output` on save and map its errors to LSP diagnostics.



* **Phase 2 Gate:** Automated LSP stdio integration tests pass for `definition`, `references`, `hover`, `documentSymbol`, `rename`, and `completion`.

---

### Phase 3: Visualizer

*Focus: Turn flat IR into visual structure without locking into one editor.*

1. **CFG & SSA Graph Engine (`ir-viz`)**:
* Extract basic blocks and terminator edges (`br`, `switch`, `ret`, `invoke`, `unreachable`) per function.
* Export to **Mermaid** (`flowchart TD`), **Graphviz DOT**, and a **self-contained HTML/SVG** view.


2. **Editor & CLI Integration**:
* CLI command: `llvm-ir-ls cfg <file.ll> --fn <name> --format mermaid|dot|html`.
* VS Code command (`LLVM IR: Show Function CFG`) rendering the output in a Webview panel.



* **Phase 3 Gate:** `llvm-ir-ls cfg` generates valid Mermaid, DOT, and HTML graphs for branching/looping `.ll` fixtures.

---

### Phase 4: AI Integration via LSP (No MCP Server)

*Focus: Make the LSP output maximally useful for AI coding agents (Antigravity, Cursor, Copilot, Zed AI) reading diagnostics and workspace signals.*

1. **AI-Grade Diagnostics**:
* Include structured **diagnostic codes** (e.g., `LLVM-SYNTAX-001`, `LLVM-VER-002`, `LLVM-SSA-003`), exact spans, expected vs. found tokens, and **relatedInformation** pointing back to the conflicting definition or version rule.


2. **Quick-Fix Code Actions (`textDocument/codeAction`)**:
* Expose deterministic fixes that both humans and AI agents can apply via LSP (e.g., *"Convert typed pointer to opaque `ptr` for LLVM 18"*, *"Replace deprecated `readnone` with `memory(none)`"*).


3. **CLI JSON Check Mode**:
* `llvm-ir-ls check <file.ll> --format json` outputs the exact same diagnostics, version info, and symbol summary as JSON so any AI agent running terminal commands can validate IR in a single call—no MCP server required.



* **Phase 4 Gate:** `llvm-ir-ls check --format json` returns structured diagnostics with error codes and suggested fixes; LSP `codeAction` applies version-migration fixes cleanly.
