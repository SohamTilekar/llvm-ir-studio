# Emacs Support for `llvm-ir-ls`

`llvm-ir-ls` can be used in GNU Emacs with either **Eglot** (built-in in Emacs 29+) or **lsp-mode**.

---

## 1. Setup with `eglot` (Recommended for Emacs 29+)

Eglot is built into modern Emacs versions and requires no external package installations.

Add the following to your `~/.emacs.d/init.el` or `~/.config/emacs/init.el`:

```elisp
(require 'eglot)

;; Associate .ll files with llvm-mode
(add-to-list 'auto-mode-alist '("\\.ll\\'" . llvm-mode))

;; Register llvm-ir-ls with eglot
(add-to-list 'eglot-server-programs
             '(llvm-mode . ("llvm-ir-ls" "lsp")))

;; Automatically start eglot when entering llvm-mode
(add-hook 'llvm-mode-hook #'eglot-ensure)
```

---

## 2. Setup with `lsp-mode`

If you use `lsp-mode`:

```elisp
(use-package lsp-mode
  :commands (lsp lsp-deferred)
  :init
  (setq lsp-keymap-prefix "C-c l")
  :config
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection '("llvm-ir-ls" "lsp"))
                    :major-modes '(llvm-mode)
                    :server-id 'llvm-ir-ls))
  :hook
  (llvm-mode . lsp-deferred))
```

---

## 3. Helper Functions

Using the bundled [`llvm-ir-ls.el`](llvm-ir-ls.el):
- `M-x llvm-ir-ls-check`: Validates the current file via `llvm-ir-ls check` and reports errors in a standard `*compilation*` buffer with jump-to-error support.
- `M-x llvm-ir-ls-cfg`: Prompts for a function name, exports the Control Flow Graph to an interactive HTML file, and opens it directly in your browser.
