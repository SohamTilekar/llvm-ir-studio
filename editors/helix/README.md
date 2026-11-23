# Helix Support for `llvm-ir-ls`

[Helix](https://helix-editor.com/) supports Language Servers out of the box with zero external plugins.

## Configuration

### 1. Ensure `llvm-ir-ls` is on your `$PATH`

Build and install `llvm-ir-ls`:
```bash
cargo install --path crates/llvm-ir-ls
```

Verify that `llvm-ir-ls` is available:
```bash
llvm-ir-ls --help
```

---

### 2. Configure `languages.toml`

Add or append the following configuration to `~/.config/helix/languages.toml` (or `%APPDATA%\helix\languages.toml` on Windows):

```toml
[[language]]
name = "llvm"
scope = "source.llvm"
injection-regex = "llvm"
file-types = ["ll"]
roots = [".llvm-ir-ls.toml", ".git"]
language-servers = ["llvm-ir-ls"]
indent = { tab-width = 2, unit = "  " }

[language-server.llvm-ir-ls]
command = "llvm-ir-ls"
args = ["lsp"]
```

Or simply append the included [`languages.toml`](languages.toml):
```bash
mkdir -p ~/.config/helix
cat editors/helix/languages.toml >> ~/.config/helix/languages.toml
```

---

### 3. Verify Health Check

Run the Helix health check:
```bash
hx --health llvm
```

You should see:
```text
Configured language server: llvm-ir-ls
Binary in $PATH: /home/.../.cargo/bin/llvm-ir-ls
```

---

## Helix Keybindings for LLVM IR

| Keybinding | Action |
| --- | --- |
| `gd` | Go to Definition (%local register or @global) |
| `gr` | Find All References |
| `space + r` | Rename Symbol |
| `space + a` | Code Action (e.g. version-migration quick-fixes) |
| `space + k` | Hover documentation (LangRef instructions and symbol types) |
| `space + d` | Open diagnostics picker |
| `[d` / `]d` | Jump to previous / next diagnostic |
