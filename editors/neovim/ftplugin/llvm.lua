-- ftplugin/llvm.lua
-- Neovim filetype plugin for LLVM IR (.ll) files
-- Automatically connects to `llvm-ir-ls` and sets up helper commands.

-- Ensure filetype is recognized as llvm
vim.bo.commentstring = "; %s"

-- 1. Start or attach the LSP server (Native Neovim 0.10+)
if vim.lsp and vim.lsp.start then
  local root_files = { '.llvm-ir-ls.toml', '.git' }
  local root_dir = vim.fs.dirname(vim.fs.find(root_files, { upward = true, path = vim.api.nvim_buf_get_name(0) })[1])
    or vim.fn.expand('%:p:h')

  vim.lsp.start({
    name = 'llvm-ir-ls',
    cmd = { 'llvm-ir-ls', 'lsp' },
    root_dir = root_dir,
    settings = {},
  }, {
    bufnr = 0,
    reuse_client = function(client, config)
      return client.name == config.name and client.config.root_dir == config.root_dir
    end,
  })
end

-- 2. Buffer Keymaps
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

-- 3. Custom Commands for IR validation and CFG visualization
vim.api.nvim_buf_create_user_command(bufnr, 'LLVMCheck', function()
  local file = vim.fn.expand('%:p')
  local cmd = string.format('llvm-ir-ls check %s', vim.fn.shellescape(file))
  local output = vim.fn.system(cmd)
  print(output)
end, { desc = 'Validate current LLVM IR file using llvm-ir-ls' })

vim.api.nvim_buf_create_user_command(bufnr, 'LLVMCFG', function(args)
  local file = vim.fn.expand('%:p')
  local fn_name = args.args ~= '' and args.args or vim.fn.expand('<cword>')
  if fn_name == '' then
    fn_name = '@main'
  elseif not vim.startswith(fn_name, '@') then
    fn_name = '@' .. fn_name
  end

  local out_file = vim.fn.tempname() .. '_cfg.html'
  local cmd = string.format('llvm-ir-ls cfg %s --fn %s --format html -o %s',
    vim.fn.shellescape(file),
    vim.fn.shellescape(fn_name),
    vim.fn.shellescape(out_file)
  )
  vim.fn.system(cmd)
  if vim.v.shell_error == 0 then
    -- Open generated HTML in default browser
    local open_cmd = vim.fn.has('mac') == 1 and 'open' or (vim.fn.has('win32') == 1 and 'start' or 'xdg-open')
    vim.fn.system(string.format('%s %s', open_cmd, vim.fn.shellescape(out_file)))
    print('Rendered CFG for ' .. fn_name .. ' -> ' .. out_file)
  else
    print('Failed to generate CFG for ' .. fn_name)
  end
end, { nargs = '?', desc = 'Generate and view HTML CFG for current function' })
