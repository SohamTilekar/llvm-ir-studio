#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:-0.1.1}"
DIST_DIR="${ROOT_DIR}/dist"

echo "=== Packaging Extensions for LLVM IR Studio (v${VERSION}) ==="
mkdir -p "${DIST_DIR}"

# 1. Package VS Code Extension (.vsix)
echo "--- Packaging VS Code Extension ---"
cd "${ROOT_DIR}/editors/vscode"
npm run build
npx @vscode/vsce package --no-dependencies --skip-license --out "${DIST_DIR}/llvm-ir-advance-${VERSION}.vsix"

# 2. Package Zed Extension (.tar.gz)
echo "--- Packaging Zed Extension ---"
cd "${ROOT_DIR}/editors/zed"
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/zed_llvm_ir_advance.wasm extension.wasm
tar -czvf "${DIST_DIR}/llvm-ir-advance-zed-${VERSION}.tar.gz" extension.toml extension.wasm languages/ README.md icon.png

echo "=== Packaging Complete ==="
ls -lh "${DIST_DIR}"
