use zed_extension_api as zed;

struct LlvmIrExtension;

impl zed::Extension for LlvmIrExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let binary_path = worktree.which("llvm-ir-ls").ok_or_else(|| {
            "llvm-ir-ls binary not found on PATH. Please compile or install llvm-ir-ls.".to_string()
        })?;

        Ok(zed::Command {
            command: binary_path,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(LlvmIrExtension);
