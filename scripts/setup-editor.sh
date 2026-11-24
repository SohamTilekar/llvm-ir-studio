#!/usr/bin/env bash
# LLVM IR Studio - Editor Auto-Configuration Script
# Detects installed text editors and configures llvm-ir-ls language server integration.
set -euo pipefail

BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
CYAN="\033[0;36m"
RESET="\033[0m"

log_info() { echo -e "${BLUE}==>${RESET} ${BOLD}$1${RESET}"; }
log_success() { echo -e "${GREEN}==>${RESET} ${BOLD}$1${RESET}"; }
log_warn() { echo -e "${YELLOW}NOTE:${RESET} $1"; }

TARGET_EDITOR="${1:-auto}"

echo -e "${BOLD}LLVM IR Studio — Editor Auto-Configurator${RESET}"
echo "--------------------------------------------------------"

# Determine base repository directory (if run locally) or fallback
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." 2>/dev/null && pwd || echo "")"

FOUND_ANY=false

# 1. Neovim Setup
setup_neovim() {
  log_info "Configuring Neovim..."
  local NVIM_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nvim"
  local FTPLUGIN_DIR="${NVIM_CONFIG_DIR}/ftplugin"
  local FTDETECT_DIR="${NVIM_CONFIG_DIR}/ftdetect"
  local LSP_DIR="${NVIM_CONFIG_DIR}/lsp"

  mkdir -p "${FTPLUGIN_DIR}" "${FTDETECT_DIR}" "${LSP_DIR}"

  # 1. ftdetect/llvm.lua (guarantees .ll -> llvm filetype)
  cat << 'EOF' > "${FTDETECT_DIR}/llvm.lua"
-- LLVM IR filetype detection for Neovim
vim.filetype.add({
  extension = {
    ll = 'llvm',
  },
})
EOF

  # 2. ftplugin/llvm.lua (native 0.10+ / 0.11+ zero-plugin setup with keymaps & commands)
  cat << 'EOF' > "${FTPLUGIN_DIR}/llvm.lua"
-- ftplugin/llvm.lua: LLVM IR filetype configuration
vim.bo.commentstring = "; %s"

-- Native Neovim LSP setup (Neovim 0.10+)
if vim.lsp and vim.lsp.start then
  local root = vim.fs.root(0, { '.llvm-ir-ls.toml', '.git' }) or vim.fn.expand('%:p:h')
  vim.lsp.start({
    name = 'llvm-ir-ls',
    cmd = { 'llvm-ir-ls', 'lsp' },
    root_dir = root,
    settings = {},
  })
end

-- Buffer Keymaps
local bufnr = vim.api.nvim_get_current_buf()
local opts = { noremap = true, silent = true, buffer = bufnr }
local map = vim.keymap.set

map('n', 'gd', vim.lsp.buf.definition, opts)
map('n', 'gr', vim.lsp.buf.references, opts)
map('n', 'K', vim.lsp.buf.hover, opts)
map('n', '<leader>rn', vim.lsp.buf.rename, opts)
map('n', '<leader>ca', vim.lsp.buf.code_action, opts)
map('n', '[d', vim.diagnostic.goto_prev, opts)
map('n', ']d', vim.diagnostic.goto_next, opts)

-- Commands: :LLVMCheck and :LLVMCFG
vim.api.nvim_buf_create_user_command(bufnr, 'LLVMCheck', function()
  local file = vim.fn.expand('%:p')
  local out = vim.fn.system(string.format('llvm-ir-ls check %s', vim.fn.shellescape(file)))
  print(out)
end, { desc = 'Validate LLVM IR using llvm-ir-ls' })

vim.api.nvim_buf_create_user_command(bufnr, 'LLVMCFG', function(args)
  local file = vim.fn.expand('%:p')
  local fn_name = args.args ~= '' and args.args or vim.fn.expand('<cword>')
  if fn_name == '' then fn_name = '@main' end
  if not vim.startswith(fn_name, '@') then fn_name = '@' .. fn_name end

  local out_file = vim.fn.tempname() .. '_cfg.html'
  local cmd = string.format('llvm-ir-ls cfg %s --fn %s --format html -o %s',
    vim.fn.shellescape(file), vim.fn.shellescape(fn_name), vim.fn.shellescape(out_file))
  vim.fn.system(cmd)
  if vim.v.shell_error == 0 then
    local open_cmd = vim.fn.has('mac') == 1 and 'open' or (vim.fn.has('win32') == 1 and 'start' or 'xdg-open')
    vim.fn.system(string.format('%s %s', open_cmd, vim.fn.shellescape(out_file)))
    print('Rendered CFG for ' .. fn_name)
  else
    print('Failed to generate CFG for ' .. fn_name)
  end
end, { nargs = '?', desc = 'Open interactive HTML CFG in browser' })
EOF

  # 3. lsp/llvm_ir_ls.lua (Neovim 0.11+ file-based LSP configuration)
  cat << 'EOF' > "${LSP_DIR}/llvm_ir_ls.lua"
---@type vim.lsp.Config
return {
  cmd = { 'llvm-ir-ls', 'lsp' },
  filetypes = { 'llvm' },
  root_markers = { '.llvm-ir-ls.toml', '.git' },
}
EOF

  log_success "Neovim configured! Files installed in ${NVIM_CONFIG_DIR}"
  echo "    - ${FTPLUGIN_DIR}/llvm.lua (Native LSP + keymaps)"
  echo "    - ${LSP_DIR}/llvm_ir_ls.lua (Neovim 0.11+ config)"
  echo "    - ${FTDETECT_DIR}/llvm.lua (Filetype mapping)"
}

# 2. Helix Setup
setup_helix() {
  log_info "Configuring Helix..."
  local HELIX_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/helix"
  local TOML="${HELIX_DIR}/languages.toml"
  mkdir -p "${HELIX_DIR}"

  if [ -f "${TOML}" ] && grep -q "llvm-ir-ls" "${TOML}"; then
    log_warn "Helix languages.toml already contains llvm-ir-ls configuration."
  else
    cat << 'EOF' >> "${TOML}"

# LLVM IR Studio Language Server
[[language]]
name = "llvm"
scope = "source.llvm"
injection-regex = "llvm"
file-types = ["ll"]
roots = [".llvm-ir-ls.toml", ".git"]
language-servers = ["llvm-ir-ls"]
indent = { tab-width = 2, unit = "  " }

[language-server.llvm-ir-ls]
command = "llvm-ir-ls"
args = ["lsp"]
EOF
    log_success "Helix configured! Added llvm-ir-ls definition to ${TOML}"
  fi
}

# 3. VS Code / Cursor / VSCodium Setup
setup_vscode() {
  if command -v code >/dev/null 2>&1; then
    log_info "Installing VS Code Extension (sohamtilekar.llvm-ir-studio)..."
    code --install-extension sohamtilekar.llvm-ir-studio || log_warn "VS Code extension install failed or already up to date."
    log_success "VS Code extension installed!"
  fi
}

setup_cursor() {
  if command -v cursor >/dev/null 2>&1; then
    log_info "Installing Cursor Extension (sohamtilekar.llvm-ir-studio)..."
    cursor --install-extension sohamtilekar.llvm-ir-studio || cursor --install-extension sohamtilekar.llvm-ir-advance || true
    log_success "Cursor extension installed!"
  fi
}

setup_vscodium() {
  if command -v codium >/dev/null 2>&1; then
    log_info "Installing VSCodium Extension (sohamtilekar.llvm-ir-advance)..."
    codium --install-extension sohamtilekar.llvm-ir-advance || true
    log_success "VSCodium extension installed!"
  fi
}

# 4. Zed Setup Instructions
setup_zed() {
  log_info "Checking Zed editor..."
  echo "To install in Zed:"
  echo "  1. Press Ctrl+Shift+P (or Cmd+Shift+P on macOS)."
  echo "  2. Run 'zed: install dev extension'."
  echo "  3. Select the 'editors/zed' directory in this repository."
}

# Run according to user target or auto-detect
run_detection() {
  echo "Scanning system for installed editors..."
  echo ""

  # Neovim
  if command -v nvim >/dev/null 2>&1 || [ -d "${XDG_CONFIG_HOME:-$HOME/.config}/nvim" ]; then
    setup_neovim
    FOUND_ANY=true
    echo ""
  fi

  # Helix
  if command -v hx >/dev/null 2>&1 || [ -d "${XDG_CONFIG_HOME:-$HOME/.config}/helix" ]; then
    setup_helix
    FOUND_ANY=true
    echo ""
  fi

  # VS Code
  if command -v code >/dev/null 2>&1; then
    setup_vscode
    FOUND_ANY=true
    echo ""
  fi

  # Cursor
  if command -v cursor >/dev/null 2>&1; then
    setup_cursor
    FOUND_ANY=true
    echo ""
  fi

  # VSCodium
  if command -v codium >/dev/null 2>&1; then
    setup_vscodium
    FOUND_ANY=true
    echo ""
  fi

  # Zed
  if command -v zed >/dev/null 2>&1 || [ -d "${HOME}/.config/zed" ]; then
    setup_zed
    FOUND_ANY=true
    echo ""
  fi

  if [ "${FOUND_ANY}" = false ]; then
    log_warn "No recognized editors were automatically found in PATH."
    echo "You can manually configure your editor by running:"
    echo "  $0 --neovim"
    echo "  $0 --helix"
    echo "  $0 --vscode"
  fi
}

case "${TARGET_EDITOR}" in
  --neovim|-n)
    setup_neovim
    ;;
  --helix|-h)
    setup_helix
    ;;
  --vscode|-v)
    setup_vscode
    ;;
  --cursor)
    setup_cursor
    ;;
  --zed|-z)
    setup_zed
    ;;
  --all|auto)
    run_detection
    ;;
  *)
    echo "Usage: $0 [--all|--neovim|--helix|--vscode|--cursor|--zed]"
    exit 1
    ;;
esac

echo "--------------------------------------------------------"
log_success "Editor configuration complete!"
