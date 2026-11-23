# LLVM IR Language Server (`llvm-ir-ls`) for Neovim

The official, native **LLVM IR Language Server & Visualizer** for Neovim:
- **Real-time Diagnostics**: Syntax errors, SSA violations, and version incompatibility warnings.
- **Go-to-Definition (`gd`)**: Jump from `%local` registers, `@global` symbols, and `br` labels to their definitions.
- **Find References (`gr`)**: Find all SSA usages and call sites across the module.
- **Rename Symbol (`<leader>rn`)**: Rename SSA registers and basic block labels in function scope.
- **Hover Documentation (`K`)**: Rich LLVM LangRef instruction documentation and type signatures.
- **Semantic Highlighting**: Accurate token classification for globals, registers, types, and labels.
- **Quick-Fix Code Actions**: Deterministic fixes for version migration.

---

## Package Installation via Mason (`mason.nvim`)

Neovim users can install and manage the language server binary automatically via [Mason](https://github.com/williamboman/mason.nvim):

```vim
:MasonInstall llvm-ir-ls
```

The package recipe and upstream submission instructions are available in [`editors/neovim/mason`](mason/README.md).

---

## Quick Start Options

### Option 1: Native Neovim (v0.10+ / v0.11+) — Zero External Plugins

Neovim 0.10+ includes built-in LSP client management with `vim.lsp.start`.

Copy [`ftplugin/llvm.lua`](ftplugin/llvm.lua) to your Neovim configuration directory:
```bash
mkdir -p ~/.config/nvim/ftplugin
cp ftplugin/llvm.lua ~/.config/nvim/ftplugin/llvm.lua
```

Whenever you open a `.ll` file, Neovim will automatically launch `llvm-ir-ls lsp` and attach it to the buffer!

---

### Option 2: `nvim-lspconfig` Setup

If you use `nvim-lspconfig`, add the following to your `init.lua`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Register llvm-ir-ls server configuration if not already present
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

-- Setup with standard LSP keybindings & capabilities (e.g. nvim-cmp)
local capabilities = vim.lsp.protocol.make_client_capabilities()
-- If using nvim-cmp:
-- capabilities = require('cmp_nvim_lsp').default_capabilities(capabilities)

lspconfig.llvm_ir_ls.setup({
  capabilities = capabilities,
  on_attach = function(client, bufnr)
    local opts = { noremap = true, silent = true, buffer = bufnr }
    local map = vim.keymap.set

    -- Navigation
    map('n', 'gd', vim.lsp.buf.definition, opts)
    map('n', 'gr', vim.lsp.buf.references, opts)
    map('n', 'K', vim.lsp.buf.hover, opts)
    map('n', '<leader>rn', vim.lsp.buf.rename, opts)
    map('n', '<leader>ca', vim.lsp.buf.code_action, opts)

    -- Diagnostic jumps
    map('n', '[d', vim.diagnostic.goto_prev, opts)
    map('n', ']d', vim.diagnostic.goto_next, opts)
    map('n', '<leader>e', vim.diagnostic.open_float, opts)
  end,
})
```

---

### Option 3: `lazy.nvim` Spec with `nvim-treesitter` & Completion

For users managing configuration with `lazy.nvim`:

```lua
return {
  -- Syntax Highlighting
  {
    "nvim-treesitter/nvim-treesitter",
    opts = function(_, opts)
      opts.ensure_installed = opts.ensure_installed or {}
      vim.list_extend(opts.ensure_installed, { "llvm" })
    end,
  },

  -- LSP Configuration
  {
    "neovim/nvim-lspconfig",
    opts = {
      servers = {
        llvm_ir_ls = {
          cmd = { "llvm-ir-ls", "lsp" },
          filetypes = { "llvm" },
          root_dir = function(fname)
            local util = require("lspconfig.util")
            return util.root_pattern(".llvm-ir-ls.toml", ".git")(fname) or util.path.dirname(fname)
          end,
        },
      },
    },
  },
}
```

---

## Built-In Commands in `ftplugin/llvm.lua`

When using the provided [`ftplugin/llvm.lua`](ftplugin/llvm.lua), the following extra commands are defined:

* `:LLVMCheck` — Runs `llvm-ir-ls check` on the current file and populates the Neovim Quickfix list (`:copen`).
* `:LLVMCFG [function_name]` — Generates the CFG for the function under the cursor (or specified name) into an HTML file and opens it in your default browser.
