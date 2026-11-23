local M = {}

---Setup function for llvm-ir-ls in Neovim
---@param opts table|nil Custom configuration options
function M.setup(opts)
  opts = opts or {}
  local lspconfig_ok, lspconfig = pcall(require, 'lspconfig')
  local configs_ok, configs = pcall(require, 'lspconfig.configs')

  if not lspconfig_ok or not configs_ok then
    vim.notify('llvm-ir-ls: nvim-lspconfig is required for llvm_ir_ls.setup()', vim.log.levels.WARN)
    return
  end

  if not configs.llvm_ir_ls then
    configs.llvm_ir_ls = {
      default_config = {
        cmd = opts.cmd or { 'llvm-ir-ls', 'lsp' },
        filetypes = { 'llvm' },
        root_dir = lspconfig.util.root_pattern('.llvm-ir-ls.toml', '.git', '.'),
        settings = opts.settings or {},
      },
    }
  end

  lspconfig.llvm_ir_ls.setup(opts)
end

return M
