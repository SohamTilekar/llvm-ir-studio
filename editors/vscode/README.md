# LLVM IR Advance for VS Code

VS Code extension for LLVM IR (`.ll`), powered by `llvm-ir-ls`.

## Capabilities

- **Language Server Protocol (LSP)**:
  - Definition lookup and Def-Use reference search
  - Diagnostic warnings with LLVM version compatibility checking (LLVM 14 to 21+)
  - Hover documentation for LLVM instructions
  - Symbol rename and autocompletions
- **CFG Visualizer**:
  - Command: `LLVM IR: Show Function CFG`
  - Interactive flowchart render with zoom, pan, and Mermaid source export
- **Diagnostics Runner**:
  - Command: `LLVM IR: Run Diagnostics Check`

## Configuration

- `llvm-ir.serverPath`: Absolute path to the `llvm-ir-ls` binary.
- `llvm-ir.version`: Target LLVM IR version (e.g. 17).
- `llvm-ir.verifyOnSave`: Run external `llvm-as` verifier on save.
