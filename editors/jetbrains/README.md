# JetBrains IDE Support for `llvm-ir-ls`
(CLion, RustRover, Fleet, IntelliJ IDEA)

Developers working in JetBrains IDEs can use `llvm-ir-ls` for LLVM IR (`.ll`) files.

---

## 1. CLion, RustRover, and IntelliJ IDEA via LSP4IJ

The easiest way to use custom Language Servers in JetBrains IDEs is with the official [LSP4IJ (Language Server Protocol for IntelliJ)](https://plugins.jetbrains.com/plugin/23257-lsp4ij) plugin.

### Setup Instructions

1. Open your JetBrains IDE (e.g. **CLion** or **RustRover**).
2. Go to **Settings / Preferences** (`Ctrl+Alt+S` or `Cmd+,`) -> **Plugins**.
3. Search for **LSP4IJ** in the Marketplace and install it. Restart the IDE.
4. Go to **Settings** -> **Languages & Frameworks** -> **Language Servers**.
5. Click **+** (Add) and configure:
   - **Server Name**: `llvm-ir-ls`
   - **Command**: `llvm-ir-ls lsp` (or path to your binary, e.g. `~/.cargo/bin/llvm-ir-ls lsp`)
   - **File Pattern**: `*.ll`
6. Click **Apply** and **OK**.

Opening any `.ll` file will now start `llvm-ir-ls` and provide:
- Syntax and version diagnostics
- Def-use navigation (Ctrl+Click / Cmd+Click)
- Renaming registers and basic blocks
- Hover documentation for LLVM IR instructions

---

## 2. JetBrains Fleet

For [JetBrains Fleet](https://www.jetbrains.com/fleet/), add the following to `.fleet/settings.json`:

```json
{
  "lsp": {
    "llvm-ir-ls": {
      "command": ["llvm-ir-ls", "lsp"],
      "filePatterns": ["*.ll"]
    }
  }
}
```

A template file is provided at [`fleet-settings.json`](fleet-settings.json).
