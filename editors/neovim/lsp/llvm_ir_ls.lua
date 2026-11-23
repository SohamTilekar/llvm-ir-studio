---@type vim.lsp.Config
return {
  cmd = { 'llvm-ir-ls', 'lsp' },
  filetypes = { 'llvm' },
  root_markers = { '.llvm-ir-ls.toml', '.git' },
}
