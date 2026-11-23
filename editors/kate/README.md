# Kate Support for `llvm-ir-ls`

[Kate](https://kate-editor.org/) (KDE Advanced Text Editor) includes a built-in LSP Client plugin that can be easily configured to use `llvm-ir-ls`.

---

## Setup Instructions

1. Ensure `llvm-ir-ls` is installed and available in `$PATH`:
   ```bash
   cargo install --path crates/llvm-ir-ls
   ```

2. Open Kate and enable the LSP Client plugin:
   - Go to **Settings** -> **Configure Kate...** -> **Plugins**.
   - Check the box for **LSP Client**.

3. Configure the Language Server:
   - Go to **Settings** -> **Configure Kate...** -> **LSP Client** -> **User Server Settings**.
   - Add the following JSON configuration snippet (or merge with your existing servers):

```json
{
  "servers": {
    "llvm": {
      "command": ["llvm-ir-ls", "lsp"],
      "rootIndicationFileNames": [".llvm-ir-ls.toml", ".git"],
      "url": "https://github.com/SohamTilekar/llvm-ir-studio",
      "highlightingModeRegex": "^LLVM.*$"
    }
  }
}
```

4. Click **Apply** and **OK**.

When you open a `.ll` file with Kate, `llvm-ir-ls` will run automatically, showing inline diagnostics, symbol outline, and hover information.
