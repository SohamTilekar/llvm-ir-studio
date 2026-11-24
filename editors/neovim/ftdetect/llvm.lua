-- LLVM IR filetype detection for Neovim (.ll -> llvm)
vim.filetype.add({
  extension = {
    ll = 'llvm',
  },
})
