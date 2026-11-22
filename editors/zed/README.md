# Zed Extension for LLVM IR (`llvm-ir-ls`)

This extension brings first-class LLVM IR language support and LSP features to [Zed](https://zed.dev).

## Features

- **Syntax Highlighting**: Tree-sitter powered syntax highlighting for `.ll` files.
- **Diagnostics & Error Recovery**: Real-time syntax and LLVM version compatibility checking (LLVM 14 through 21+).
- **Navigation**:
  - Go to Definition for `%registers`, `@functions`, `@globals`, `%types`, and basic block labels.
  - Find all references & SSA def-use chains.
- **Comprehension & Hover**:
  - LLVM LangRef hover documentation for all IR instructions.
  - Type definitions and symbol summaries.
  - Document Symbols outline (Module -> Types, Globals, Functions -> Basic Blocks).
- **Editing**:
  - Autocompletion for instructions, types, and in-scope registers/labels.
  - Safe local SSA register and basic block label renaming.
  - Quick-fix code actions for version migrations (e.g. converting `i32*` to `ptr` for LLVM 15+).

## Installation in Zed

1. Build or install `llvm-ir-ls`:
   ```bash
   cargo install --path crates/llvm-ir-ls
   ```
   Ensure `llvm-ir-ls` is in your `$PATH`.

2. Install as a dev extension in Zed:
   - Open Zed and press `ctrl-shift-p` / `cmd-shift-p`.
   - Run `zed: install dev extension`.
   - Select the `editors/zed` directory from this repository.
