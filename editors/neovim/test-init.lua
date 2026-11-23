-- Standalone test configuration for testing llvm-ir-ls in Neovim
-- Run with:
--   nvim -u editors/neovim/test-init.lua tests/fixtures/syntax_broken.ll

-- 1. Basic editor options
vim.opt.number = true
vim.opt.signcolumn = 'yes'
vim.opt.termguicolors = true
vim.g.mapleader = ' '

-- 2. Detect path to llvm-ir-ls binary (prefer target/release)
local root_dir = vim.fn.fnamemodify(vim.fn.resolve(vim.fn.expand('<sfile>:p')), ':h:h:h')
local bin_path = root_dir .. '/target/release/llvm-ir-ls'
if vim.fn.filereadable(bin_path) == 0 then
  bin_path = root_dir .. '/target/debug/llvm-ir-ls'
end

-- 3. Filetype detection for .ll
vim.filetype.add({
  extension = {
    ll = 'llvm',
  },
})

-- 4. Autocommand to start llvm-ir-ls on LLVM IR files
vim.api.nvim_create_autocmd('FileType', {
  pattern = 'llvm',
  callback = function(ev)
    local client_id = vim.lsp.start({
      name = 'llvm-ir-ls',
      cmd = { bin_path, 'lsp' },
      root_dir = root_dir,
      settings = {},
    })

    if client_id then
      vim.notify('⚡ LLVM IR Advance (llvm-ir-ls) started and attached to buffer!', vim.log.levels.INFO)
    else
      vim.notify('❌ Failed to start llvm-ir-ls at ' .. bin_path, vim.log.levels.ERROR)
    end

    -- Buffer Keymaps
    local opts = { buffer = ev.buf, silent = true, noremap = true }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, opts)
    vim.keymap.set('n', ']d', vim.diagnostic.goto_next, opts)
    vim.keymap.set('n', '<leader>e', vim.diagnostic.open_float, opts)
  end,
})

-- 5. Helper commands
vim.api.nvim_create_user_command('LLVMCheck', function()
  local file = vim.fn.expand('%:p')
  local out = vim.fn.system({ bin_path, 'check', file })
  print(out)
end, { desc = 'Validate current LLVM IR file' })

vim.api.nvim_create_user_command('LLVMCFG', function(args)
  local file = vim.fn.expand('%:p')
  local fn_name = args.args ~= '' and args.args or vim.fn.expand('<cword>')
  if fn_name == '' then fn_name = '@main' end
  if not vim.startswith(fn_name, '@') then fn_name = '@' .. fn_name end

  local out_file = vim.fn.tempname() .. '_cfg.html'
  local res = vim.fn.system({ bin_path, 'cfg', file, '--fn', fn_name, '--format', 'html', '-o', out_file })
  if vim.v.shell_error == 0 then
    local open_cmd = vim.fn.has('mac') == 1 and 'open' or (vim.fn.has('win32') == 1 and 'start' or 'xdg-open')
    vim.fn.system({ open_cmd, out_file })
    print('Rendered CFG for ' .. fn_name .. ' -> ' .. out_file)
  else
    print('Failed to generate CFG: ' .. res)
  end
end, { nargs = '?', desc = 'Generate and view HTML CFG' })
