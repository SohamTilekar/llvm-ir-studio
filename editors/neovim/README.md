# LLVM IR Language Server (`llvm-ir-ls`) for Neovim

Native LLVM IR Language Server integration for Neovim (v0.10+ / v0.11+).

---

## 🚀 Quick Setup (Automated)

Run the auto-setup script from the root of this repository:

```bash
./setup-editor.sh --neovim
```

Or via curl:

```bash
curl -fsSL https://raw.githubusercontent.com/SohamTilekar/llvm-ir-studio/main/setup-editor.sh | bash -s -- --neovim
```

This automatically installs the filetype detection and native LSP client configurations directly into your `~/.config/nvim/` directory.

---

## 🛠 Manual Configuration

### Option 1: Native Neovim (v0.10+ / v0.11+) — Recommended (Zero Plugins)

Neovim has built-in LSP client management. You do not need any external plugins!

1. **Filetype Detection**: Ensure `~/.config/nvim/ftdetect/llvm.lua` exists:
   ```lua
   vim.filetype.add({ extension = { ll = 'llvm' } })
   ```

2. **LSP Autostart**: Copy [`ftplugin/llvm.lua`](ftplugin/llvm.lua) to `~/.config/nvim/ftplugin/llvm.lua`:
   ```bash
   mkdir -p ~/.config/nvim/ftplugin
   cp ftplugin/llvm.lua ~/.config/nvim/ftplugin/llvm.lua
   ```

Whenever you open any `.ll` file:
- Neovim automatically attaches `llvm-ir-ls`.
- Pre-wires standard keymaps: `gd` (definition), `gr` (references), `K` (hover), `<leader>rn` (rename), `<leader>ca` (code action).
- Provides buffer commands `:LLVMCheck` and `:LLVMCFG`.

---

### Option 2: Neovim 0.11+ Native LSP Config

If you are using Neovim 0.11+, you can use the native file-based LSP configuration:

1. Copy [`lsp/llvm_ir_ls.lua`](lsp/llvm_ir_ls.lua) to `~/.config/nvim/lsp/llvm_ir_ls.lua`:
   ```bash
   mkdir -p ~/.config/nvim/lsp
   cp lsp/llvm_ir_ls.lua ~/.config/nvim/lsp/llvm_ir_ls.lua
   ```

2. Enable the server in your `init.lua`:
   ```lua
   vim.lsp.enable('llvm_ir_ls')
   ```

---

### Option 3: `nvim-lspconfig` (Neovim 0.10 and earlier)

If you use `nvim-lspconfig`, add the following to your `init.lua`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.llvm_ir_ls then
  configs.llvm_ir_ls = {
    default_config = {
      cmd = { 'llvm-ir-ls', 'lsp' },
      filetypes = { 'llvm' },
      root_dir = lspconfig.util.root_pattern('.llvm-ir-ls.toml', '.git', '.'),
      settings = {},
    },
  }
end

lspconfig.llvm_ir_ls.setup({})
```

---

## ⌨️ Features & Commands

Once attached to a `.ll` buffer:
- **`gd`**: Go to definition of SSA register or `@global` symbol.
- **`gr`**: Find all SSA usages and call references.
- **`K`**: Hover for LLVM LangRef instruction syntax and documentation.
- **`<leader>rn`**: In-function SSA rename.
- **`:LLVMCheck`**: Run IR validation and print diagnostics.
- **`:LLVMCFG`**: Generate and open the function's Control Flow Graph in your default web browser.
