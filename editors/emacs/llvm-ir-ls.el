;;; llvm-ir-ls.el --- llvm-ir-ls Language Server integration for GNU Emacs -*- lexical-binding: t; -*-

;; Author: llvm-ir-ls contributors
;; Version: 0.1.0
;; Keywords: languages, tools, llvm, lsp

;;; Commentary:
;; This package configures `llvm-ir-ls` for Emacs using either `eglot` (built-in in Emacs 29+)
;; or `lsp-mode`.

;;; Code:

(defgroup llvm-ir-ls nil
  "LLVM IR Language Server integration."
  :group 'languages
  :prefix "llvm-ir-ls-")

(defcustom llvm-ir-ls-executable "llvm-ir-ls"
  "Path to the llvm-ir-ls binary."
  :type 'string
  :group 'llvm-ir-ls)

;;;###autoload
(defun llvm-ir-ls-setup-eglot ()
  "Configure `eglot` to use `llvm-ir-ls` for `llvm-mode`."
  (interactive)
  (when (fboundp 'eglot-server-programs)
    (add-to-list 'eglot-server-programs
                 `(llvm-mode . (,llvm-ir-ls-executable "lsp")))))

;;;###autoload
(defun llvm-ir-ls-setup-lsp-mode ()
  "Configure `lsp-mode` to use `llvm-ir-ls` for `llvm-mode`."
  (interactive)
  (when (featurep 'lsp-mode)
    (lsp-register-client
     (make-lsp-client :new-connection (lsp-stdio-connection (list llvm-ir-ls-executable "lsp"))
                      :major-modes '(llvm-mode)
                      :server-id 'llvm-ir-ls))))

;;;###autoload
(defun llvm-ir-ls-check ()
  "Run `llvm-ir-ls check` on the current buffer file."
  (interactive)
  (let ((file (buffer-file-name)))
    (if (not file)
        (user-error "Current buffer has no associated file")
      (compile (format "%s check %s" (shell-quote-argument llvm-ir-ls-executable) (shell-quote-argument file))))))

;;;###autoload
(defun llvm-ir-ls-cfg (func-name)
  "Generate and display HTML CFG for FUNC-NAME in the current file."
  (interactive (list (read-string "Function name to visualize (e.g. @main): "
                                  (thing-at-point 'symbol t))))
  (let* ((file (buffer-file-name))
         (out-file (make-temp-file "llvm_ir_cfg_" nil ".html")))
    (if (not file)
        (user-error "Current buffer has no associated file")
      (let ((fn (if (string-prefix-p "@" func-name) func-name (concat "@" func-name))))
        (call-process llvm-ir-ls-executable nil nil nil
                      "cfg" file "--fn" fn "--format" "html" "-o" out-file)
        (browse-url-of-file out-file)
        (message "Rendered CFG for %s -> %s" fn out-file)))))

(provide 'llvm-ir-ls)
;;; llvm-ir-ls.el ends here
