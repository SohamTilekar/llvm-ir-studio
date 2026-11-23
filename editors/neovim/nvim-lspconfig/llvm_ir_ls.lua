local util = require 'lspconfig.util'

return {
  default_config = {
    cmd = { 'llvm-ir-ls', 'lsp' },
    filetypes = { 'llvm' },
    root_dir = util.root_pattern('.llvm-ir-ls.toml', '.git'),
    single_file_support = true,
  },
  docs = {
    description = [[
https://github.com/SohamTilekar/llvm-ir-studio

LLVM IR Language Server & Visualizer (`llvm-ir-ls`).

Features:
- Pure Rust implementation with zero libLLVM C++ linking.
- LLVM IR Version-Aware (LLVM 14 to 21+ syntax & compatibility checking).
- Real-time syntax and semantic diagnostics with machine-readable error codes.
- SSA Def-Use analysis: Go-to-definition, find all references, symbol rename.
- Control Flow Graph (CFG) visualizer for basic blocks and branch targets.
- LangRef instruction hover documentation.
]],
  },
}
