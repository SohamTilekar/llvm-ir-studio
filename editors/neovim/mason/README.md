# Publishing LLVM IR Language Server to Mason Registry (`mason.nvim`)

This guide outlines how to submit the **LLVM IR Language Server** (`llvm-ir-ls`) to the official [`mason-org/mason-registry`](https://github.com/mason-org/mason-registry) so any Neovim user worldwide can install it via `:Mason` or `:MasonInstall llvm-ir-ls`.

---

## 1. Package Manifest

The package manifest is prepared at [`package.yaml`](package.yaml):
- **Package Name**: `llvm-ir-ls`
- **Alias**: `llvm-ir-advance`
- **Binary symlink**: Exposes `llvm-ir-ls` on the user's `$PATH`.
- **Prebuilt Targets**:
  - `linux_x64_gnu` (`x86_64-unknown-linux-gnu`)
  - `linux_arm64_gnu` (`aarch64-unknown-linux-gnu`)
  - `darwin_arm64` (`aarch64-apple-darwin` Apple Silicon)
  - `darwin_x64` (`x86_64-apple-darwin` Intel Mac)
  - `win_x64` (`x86_64-pc-windows-msvc`)

---

## 2. Step-by-Step Submission Process

### Step 1: Fork and Clone `mason-org/mason-registry`
```bash
git clone https://github.com/<YOUR_GITHUB_USERNAME>/mason-registry.git
cd mason-registry
```

### Step 2: Create a Branch
```bash
git checkout -b add-llvm-ir-ls
```

### Step 3: Add the Package Manifest
Create the directory `packages/llvm-ir-ls/` and copy [`package.yaml`](package.yaml) into it:
```bash
mkdir -p packages/llvm-ir-ls
cp /path/to/llvm-ir-studio/editors/neovim/mason/package.yaml packages/llvm-ir-ls/package.yaml
```

### Step 4: Validate and Test the Package
Mason Registry uses Node.js tooling to validate the schema:
```bash
# Install registry dependencies
npm install

# Validate package definitions against registry schema
npm run check packages/llvm-ir-ls/package.yaml
```

You can also test installing it directly inside Neovim by testing with Mason:
```vim
:MasonInstall --target=linux_x64 packages/llvm-ir-ls/package.yaml
```

### Step 5: Commit and Open a Pull Request
```bash
git add packages/llvm-ir-ls/package.yaml
git commit -m "feat(packages): add llvm-ir-ls (LLVM IR Language Server)"
git push origin add-llvm-ir-ls
```

Go to `https://github.com/mason-org/mason-registry` and click **Compare & pull request**.

---

## 3. Pairing with `mason-lspconfig.nvim` & `nvim-lspconfig`

Once accepted into `mason-registry`:
1. Mason automatically handles automated updates whenever a new version tag (e.g. `v0.1.1`, `v0.2.0`) is published on `SohamTilekar/llvm-ir-studio`.
2. Users can simply add to their Neovim config:
```lua
require("mason").setup()
require("mason-lspconfig").setup({
  ensure_installed = { "llvm_ir_ls" }
})
```
