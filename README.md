# llvm-ir-ls

> Lightweight, version-aware, native LLVM IR Language Server & Visualizer in Rust.

`llvm-ir-ls` provides universal LSP support across editors (Zed, VS Code, Neovim, Helix) for LLVM Intermediate Representation (`.ll`) files with:
- **Zero `libLLVM` C++ linking**: Pure Rust, fast builds, low memory footprint.
- **LLVM Version Awareness**: Full version profiles for LLVM 14 through 21+ (opaque `ptr` vs typed pointers `i32*`, deprecated attributes, memory syntax).
- **Error-tolerant incremental parsing**: Tree-sitter powered CST with live error recovery.
- **AI-Grade LSP Output**: Machine-readable diagnostic error codes (`LLVM-SYNTAX-001`, `LLVM-VER-001`, `LLVM-SSA-001`, `LLVM-CFG-001`), related information, and quick-fix code actions.
- **CFG Visualizer**: Export control flow graphs to Mermaid (`flowchart TD`), Graphviz DOT, or interactive HTML.

---

## Workspace Architecture

```text
llvm-ir-ls/
├── Cargo.toml
├── crates/
│   ├── ir-core/          # Tree-sitter parser, ropey document store, UTF-16 mapper
│   ├── ir-version/       # LLVM IR version profiles (14–21+), version detector, compatibility rules
│   ├── ir-analysis/      # Syntax/semantic diagnostics, symbol table, SSA def-use, CFG
│   ├── ir-docs/          # Version-tagged LLVM LangRef hover docs
│   ├── ir-viz/           # CFG / SSA exporter (Mermaid, DOT, HTML)
│   ├── ir-lsp/           # tower-lsp JSON-RPC handlers
│   └── llvm-ir-ls/       # CLI binary (`lsp`, `check`, `cfg`)
├── editors/
│   └── zed/              # Zed extension for LLVM IR
└── tests/
    └── fixtures/         # Versioned test fixtures (valid, broken, CFG)
```

---

## Installation & Build

Build the workspace binary:

```bash
cargo build --release
```

The compiled binary will be located at `target/release/llvm-ir-ls`.

---

## CLI Usage

### 1. Run as LSP Server

```bash
llvm-ir-ls lsp
# or simply
llvm-ir-ls
```

### 2. Check File Diagnostics (Human & AI-Friendly)

Human-readable text output:
```bash
llvm-ir-ls check tests/fixtures/valid_typed.ll --version 18
```

Machine-readable JSON output for AI coding agents:
```bash
llvm-ir-ls check tests/fixtures/valid_opaque.ll --format json
```

### 3. CFG Visualization

Export Mermaid graph:
```bash
llvm-ir-ls cfg tests/fixtures/branching_cfg.ll --format mermaid
```

Export Graphviz DOT:
```bash
llvm-ir-ls cfg tests/fixtures/branching_cfg.ll --format dot
```

Export self-contained HTML:
```bash
llvm-ir-ls cfg tests/fixtures/branching_cfg.ll --format html -o cfg.html
```

---

## Editor Integration

### Zed

See [editors/zed/README.md](file:///mnt/soham/soham_code/llvm-ir-studio/editors/zed/README.md).
Install as dev extension in Zed via `zed: install dev extension` pointing to `editors/zed`.

---

## Running Tests

```bash
cargo test
```
