# LSP-llvm-ir: Advanced LLVM IR Language Server for Sublime Text

`LSP-llvm-ir` is the official helper package providing seamless integration for the **Advanced LLVM IR Language Server & Visualizer** (`llvm-ir-ls`) in [Sublime Text](https://www.sublimetext.com/).

It integrates with the Sublime Text [LSP](https://github.com/sublimelsp/LSP) plugin ecosystem to deliver first-class LLVM IR tooling.

---

## Capabilities

- **Real-time Diagnostics**: Syntax errors, SSA violations, and version incompatibility warnings (LLVM 14 to 21+).
- **Go to Definition (`F12`)**: Jump directly to instruction definitions of registers (`%0`, `%val`) and functions (`@main`).
- **Find References (`Shift+F12`)**: Find all SSA usages across the module.
- **Hover Documentation**: Hover over any LLVM instruction to view syntax, operands, and LLVM LangRef documentation.
- **Symbol Rename**: Scope-safe renaming of registers and basic block labels.
- **Code Actions**: Quick-fix code actions for version migration (e.g. converting typed `i32*` pointers to opaque `ptr`).

---

## Installation

### Prerequisites

1. Install **[Package Control](https://packagecontrol.io/)** in Sublime Text.
2. Install the **`LSP`** package:
   - `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (macOS) -> `Package Control: Install Package`.
   - Search for **`LSP`** and press Enter.
3. Install an LLVM syntax highlighter (e.g., search for **`LLVM`** in Package Control) so `.ll` files have scope `source.llvm`.
4. Ensure `llvm-ir-ls` is installed on your system `$PATH`:
   ```bash
   cargo install --git https://github.com/SohamTilekar/llvm-ir-studio.git llvm-ir-ls
   # or download prebuilt binary from GitHub Releases
   ```

---

### Method A: Install via Package Control (Recommended once published)

1. Open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`).
2. Run `Package Control: Install Package`.
3. Search for `LSP-llvm-ir` and press Enter.

---

### Method B: Manual Installation (Zero waiting)

1. Clone or copy this directory directly into your Sublime Text `Packages/` folder:
   ```bash
   # Linux
   cd ~/.config/sublime-text/Packages/
   git clone https://github.com/SohamTilekar/LSP-llvm-ir.git

   # macOS
   cd ~/Library/Application\ Support/Sublime\ Text/Packages/
   git clone https://github.com/SohamTilekar/LSP-llvm-ir.git

   # Windows (PowerShell)
   cd "$env:APPDATA\Sublime Text\Packages"
   git clone https://github.com/SohamTilekar/LSP-llvm-ir.git
   ```
2. Restart Sublime Text or open any `.ll` file. `LSP-llvm-ir` activates automatically!

---

## How to Submit to Package Control Channel

To make `LSP-llvm-ir` discoverable in the official Sublime Text package directory:

1. **Create GitHub Repo**:
   Create a standalone repository `https://github.com/SohamTilekar/LSP-llvm-ir` containing the files in this folder (`plugin.py`, `sublime-package.json`, `LSP-llvm-ir.sublime-settings`, `Main.sublime-menu`, `README.md`).
2. **Create a Tag**:
   Create a Git tag for the first release:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. **Fork `sublimehq/package_control_channel`**:
   Fork [https://github.com/sublimehq/package_control_channel](https://github.com/sublimehq/package_control_channel).
4. **Add Package Entry**:
   Open `repository/l.json` and insert the definition from [`package-control.json`](package-control.json):
   ```json
   {
     "name": "LSP-llvm-ir",
     "details": "https://github.com/SohamTilekar/LSP-llvm-ir",
     "description": "Advanced LLVM IR Language Server & Visualizer for Sublime Text",
     "homepage": "https://github.com/SohamTilekar/llvm-ir-studio",
     "author": "Soham Tilekar",
     "labels": [
       "lsp",
       "llvm",
       "language-server",
       "compiler",
       "diagnostics"
     ],
     "releases": [
       {
         "sublime_text": ">=4000",
         "tags": true
       }
     ]
   }
   ```
5. **Open Pull Request**:
   Commit and open a Pull Request against `sublimehq/package_control_channel`.
