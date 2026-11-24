# Editor Extensions & Registry Distribution Tracker

This document tracks the publication, review, and distribution status of **LLVM IR Studio** / **LLVM IR Advance** (`llvm-ir-ls`) across all supported text editors, IDEs, and package managers.

Last Updated: November 2026

---

## 📊 Status Matrix

| Editor / Ecosystem | Package / Display Name | Target Registry / Platform | Status | Link / PR | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **VS Code & Cursor** | `LLVM IR Studio` (`SohamTilekar.llvm-ir-studio`) | [Visual Studio Marketplace](https://marketplace.visualstudio.com/) | 🟡 **Published (Under Review)** | [Marketplace Link](https://marketplace.visualstudio.com/items?itemName=SohamTilekar.llvm-ir-studio) | Uploaded & available; publisher verification / review pending. |
| **Open VSX (VSCodium, Gitpod)** | `LLVM IR Advance` (`sohamtilekar/llvm-ir-advance`) | [Open VSX Registry](https://open-vsx.org/) | 🟡 **Published (Under Review)** | [Open VSX Link](https://open-vsx.org/extension/sohamtilekar/llvm-ir-advance) | Uploaded & available; review in progress. |
| **Zed** | `LLVM IR Advance` (`llvm-ir-advance`) | [Zed Extensions Registry](https://github.com/zed-industries/extensions) | 🔵 **In Progress** | PR to `zed-industries/extensions` | WASM extension with automatic binary downloader built and tested. |
| **Neovim (Mason)** | `LLVM IR Language Server` (`llvm-ir-ls`) | [Mason Registry](https://github.com/mason-org/mason-registry) | 🔴 **Declined / Ineligible** | `packages/llvm-ir-ls` | Registry inclusion not approved by maintainers due to external repository star/download thresholds. Users install via official `install.sh`, Cargo, or GitHub Releases. |
| **Neovim (Upstream)** | `llvm_ir_ls` | [neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) | 🔵 **In Progress (PR Ready)** | [Open PR](https://github.com/SohamTilekar/nvim-lspconfig/pull/new/add-llvm-ir-ls) | Branch `add-llvm-ir-ls` pushed to fork; ready to open PR. |
| **Neovim (Native / Local)** | `llvm-ir-ls` | Built-in Neovim `vim.lsp.start` | 🟢 **Complete** | [`editors/neovim/ftplugin`](editors/neovim/ftplugin) | Zero-plugin native setup tested and fully working. |
| **Sublime Text** | `LSP-llvm-ir` | [Sublime Package Control](https://packagecontrol.io/) | 🔵 **In Progress (PR Ready)** | [Open PR](https://github.com/SohamTilekar/package_control_channel/pull/new/add-lsp-llvm-ir) | Branch `add-lsp-llvm-ir` pushed to fork; ready to open PR. |
| **GNU Emacs** | `llvm-ir-ls` / `llvm-ir-advance` | [MELPA](https://melpa.org/) | ⚪ **Planned** | [`editors/emacs`](editors/emacs) | Recipe for `llvm-ir-ls.el` ready for MELPA submission. |
| **Helix** | `llvm-ir-ls` | [helix-editor/helix](https://github.com/helix-editor/helix) | ⚪ **Planned** | [`editors/helix`](editors/helix) | Upstream PR to Helix's default `languages.toml`. |
| **JetBrains (CLion/Fleet)** | `llvm-ir-ls` | JetBrains LSP4IJ & Fleet Settings | 🟢 **Configured** | [`editors/jetbrains`](editors/jetbrains) | Configuration guides and Fleet JSON settings provided. |
| **Kate (KDE)** | `llvm-ir-ls` | Kate Built-in LSP Client | 🟢 **Configured** | [`editors/kate`](editors/kate) | JSON server definition provided. |

---

## 📌 Legend
- 🟢 **Complete**: Ready, tested, and usable right now.
- 🟡 **Published (Under Review)**: Uploaded to official store; undergoing marketplace approval or publisher review.
- 🔵 **In Progress**: Extension built, packaging complete, PR/submission active.
- 🔴 **On Hold**: Paused (e.g., waiting for project stars, release maturity, or upstream prerequisites).
- ⚪ **Planned**: Files and recipes prepared; waiting for next distribution phase.

---

## 📝 Detailed Editor Breakdown

### 1. Visual Studio Code & Open VSX (Cursor, VSCodium, Gitpod)
- **VS Code Marketplace**: [`SohamTilekar.llvm-ir-studio`](https://marketplace.visualstudio.com/items?itemName=SohamTilekar.llvm-ir-studio)
- **Open VSX Registry**: [`sohamtilekar/llvm-ir-advance`](https://open-vsx.org/extension/sohamtilekar/llvm-ir-advance)
- **Package File**: `llvm-ir-studio-${VERSION}.vsix` (available from GitHub Releases & Marketplaces)
- **Current Status**:
  - **VS Code Marketplace**: 🟡 Uploaded and live; publisher verification / review in progress.
  - **Open VSX Registry**: 🟡 Uploaded and live; review in progress.

### 2. Zed Editor
- **Display Name**: `LLVM IR Advance`
- **Identifier**: `llvm-ir-advance`
- **Current Status**: **In Progress**.
- **Artifacts**:
  - `extension.toml` configured with description, version, and grammar.
  - Rust/WASM extension compiled to `extension.wasm`.
  - Tarball: `dist/llvm-ir-advance-zed-0.1.0.tar.gz`.
- **Next Action**: Complete submission PR to `zed-industries/extensions`.

### 3. Neovim
- **Primary Package Name**: `llvm-ir-ls`
- **Display Name**: `LLVM IR Language Server`
- **Current Status**:
  - **Local Native Support**: 🟢 **Working**. Tested and verified with [`editors/neovim/test-init.lua`](editors/neovim/test-init.lua) and [`editors/neovim/ftplugin/llvm.lua`](editors/neovim/ftplugin/llvm.lua).
  - **Mason Registry**: 🔴 **Declined / Ineligible**. Inclusion was not approved due to Mason's external star/usage threshold. Manifest is preserved in [`editors/neovim/mason/package.yaml`](editors/neovim/mason/package.yaml) for future re-evaluation. Binary is distributed via official install script, GitHub Releases, and Cargo.
  - **`nvim-lspconfig`**: Upstream server config is ready at [`editors/neovim/nvim-lspconfig/llvm_ir_ls.lua`](editors/neovim/nvim-lspconfig/llvm_ir_ls.lua).

### 4. Sublime Text
- **Package Name**: `LSP-llvm-ir-advance`
- **Current Status**: ⚪ **Planned**.
- **Configuration**: Settings prepared in [`editors/sublime/LSP-llvm-ir.sublime-settings`](editors/sublime/LSP-llvm-ir.sublime-settings).
- **Target**: Submit to `wbond/package_control_channel`.

### 5. GNU Emacs
- **Package Name**: `llvm-ir-ls`
- **Current Status**: ⚪ **Planned**.
- **Elisp Module**: [`editors/emacs/llvm-ir-ls.el`](editors/emacs/llvm-ir-ls.el) (supports built-in `eglot` and `lsp-mode`).
- **Target**: Submit recipe to MELPA repository.

### 6. Helix
- **Current Status**: ⚪ **Planned**.
- **Configuration**: [`editors/helix/languages.toml`](editors/helix/languages.toml).
- **Target**: Submit PR to Helix's official repository `languages.toml`.

---

## 🎯 Next Priority Steps
1. [ ] **Zed**: Finalize and track PR in `zed-industries/extensions`.
2. [ ] **VS Code Marketplace**: Monitor verification / review status until approved.
3. [ ] **Open VSX**: Monitor review status until verified.
4. [ ] **`nvim-lspconfig`**: Open PR on `neovim/nvim-lspconfig` to enable standard setup for Neovim users.
5. [ ] Revisit **Mason Registry** once stars/activity thresholds are reached.
