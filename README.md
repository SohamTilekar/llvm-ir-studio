# LLVM IR Studio (`llvm-ir-ls`)

> Lightweight, version-aware, native LLVM IR Language Server & Visualizer written in Rust.

`llvm-ir-ls` brings first-class developer tooling for LLVM Intermediate Representation (`.ll`) files across modern editors with:
- **Zero `libLLVM` C++ linking**: Pure Rust implementation with instantaneous startup and tiny memory footprint.
- **LLVM Version Awareness**: Tracks syntax evolution across LLVM 14 to 21+ (opaque `ptr` vs typed pointers `i32*`, explicit `getelementptr` types, deprecated attributes like `readnone` → `memory(...)`).
- **Incremental & Error-Tolerant**: Powered by Tree-sitter for uninterrupted syntax parsing even on partially written or broken IR files.
- **SSA Def-Use Analysis**: Instant Go-to-Definition, Find All References, and in-function SSA register/basic block renaming.
- **Control Flow Graph (CFG) Visualizer**: Interactive flowcharts rendered directly inside VS Code or exported to Mermaid, Graphviz DOT, and standalone HTML/SVG.
- **AI Coding Agent Ready**: Machine-readable diagnostic codes (`LLVM-SYNTAX-001`, `LLVM-VER-001`, `LLVM-SSA-001`, `LLVM-CFG-001`), related information spans, and quick-fix code actions.

---

## Editor Support

### 1. Visual Studio Code & Cursor (`llvm-ir-advance`)

The VS Code extension is located in [`editors/vscode`](file:///mnt/soham/soham_code/llvm-ir-studio/editors/vscode).

#### Key Capabilities
- **Full LSP Support**: Diagnostics, hover definitions, code completions, symbol rename, and document symbols.
- **Smart Prefix Completion**: Eliminates prefix duplication when completing `@globals`, `%registers`, and `.labels`.
- **Interactive CFG Webview**: Right-click or run `LLVM IR: Show Function CFG` to view and pan/zoom the function's control flow graph in a side panel.
- **Automatic Language Server Provisioning**: Detects local `llvm-ir-ls` binaries or downloads the matching pre-built release automatically from GitHub Releases.

#### Installation
- **Install from VSIX**:
  ```bash
  code --install-extension editors/vscode/llvm-ir-advance-0.1.0.vsix
  ```
- **Visual Studio Marketplace**: Search for `LLVM IR Advance` (`sohamtilekar.llvm-ir-advance`).

---

### 2. Zed (`llvm-ir-advance`)

The Zed extension is located in [`editors/zed`](file:///mnt/soham/soham_code/llvm-ir-studio/editors/zed).

#### Key Capabilities
- Tree-sitter powered syntax highlighting for `.ll` files.
- Real-time version checking and diagnostics.
- Go to Definition & Find References across SSA registers and basic block labels.
- Auto-downloading binary installer powered by Zed's Extension API.

#### Installation
1. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) in Zed.
2. Run `zed: install dev extension`.
3. Choose the [`editors/zed`](file:///mnt/soham/soham_code/llvm-ir-studio/editors/zed) folder.

---

### 3. Neovim

Add the following to your `init.lua` (requires `llvm-ir-ls` on your `$PATH`):

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.llvm_ir_ls then
  configs.llvm_ir_ls = {
    default_config = {
      cmd = { 'llvm-ir-ls', 'lsp' },
      filetypes = { 'llvm' },
      root_dir = lspconfig.util.root_pattern('.git', '.llvm-ir-ls.toml'),
      settings = {},
    },
  }
end

lspconfig.llvm_ir_ls.setup{}
```

---

### 4. Helix

Add `llvm-ir-ls` to your `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "llvm"
scope = "source.llvm"
injection-regex = "llvm"
file-types = ["ll"]
roots = [".llvm-ir-ls.toml", ".git"]
language-servers = ["llvm-ir-ls"]

[language-server.llvm-ir-ls]
command = "llvm-ir-ls"
args = ["lsp"]
```

---

## Workspace Layout

```text
llvm-ir-studio/
├── Cargo.toml
├── crates/
│   ├── ir-core/          # Incremental Tree-sitter parser, document store, UTF-16 position mapper
│   ├── ir-version/       # Version profiles (LLVM 14–21+), compatibility rules, heuristic detection
│   ├── ir-analysis/      # Syntax & semantic analysis, symbol table, SSA def-use chains, CFG builder
│   ├── ir-docs/          # LLVM LangRef instruction hover documentation
│   ├── ir-viz/           # CFG visualizer (Mermaid, Graphviz DOT, interactive HTML)
│   ├── ir-lsp/           # tower-lsp implementation (diagnostics, semantic tokens, actions, completions)
│   └── llvm-ir-ls/       # Unified CLI binary (`lsp`, `check`, `cfg`)
├── editors/
│   ├── vscode/           # VS Code extension (TypeScript LSP client, TextMate grammar, Webview CFG)
│   └── zed/              # Zed Rust/WASM extension with auto-downloader
└── tests/
    └── fixtures/         # Multi-version LLVM IR fixtures
```

---

## CLI Usage

### 1. Launch Language Server (LSP Mode)

```bash
llvm-ir-ls lsp
# or simply
llvm-ir-ls
```

### 2. IR Validation & Diagnostics

Human-readable terminal output:
```bash
llvm-ir-ls check file.ll --version 18
```

Machine-readable JSON output for AI coding agents and CI:
```bash
llvm-ir-ls check file.ll --format json
```

### 3. CFG Visualization

Export Mermaid diagram:
```bash
llvm-ir-ls cfg file.ll --fn @main --format mermaid
```

Export Graphviz DOT file:
```bash
llvm-ir-ls cfg file.ll --fn @main --format dot -o cfg.dot
```

Generate interactive HTML viewer:
```bash
llvm-ir-ls cfg file.ll --fn @main --format html -o cfg.html
```

---

## Building from Source

Prerequisites: Rust stable toolchain (`cargo`).

```bash
# Clone the repository
git clone https://github.com/SohamTilekar/llvm-ir-studio.git
cd llvm-ir-studio

# Build optimized binary
cargo build --release --bin llvm-ir-ls

# Install binary to ~/.cargo/bin or ~/.local/bin
cargo install --path crates/llvm-ir-ls
```

Run test suite:
```bash
cargo test
```
