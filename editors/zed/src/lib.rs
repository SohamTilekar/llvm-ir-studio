use std::fs;
use zed_extension_api as zed;

const GITHUB_REPO: &str = "SohamTilekar/llvm-ir-studio";

struct LlvmIrExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for LlvmIrExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let binary_path = self.language_server_binary(language_server_id, worktree)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

impl LlvmIrExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        // 1. If we already resolved and cached binary path in memory, verify and use it
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        // 2. Check if llvm-ir-ls is installed globally on the user's PATH
        if let Some(path) = worktree.which("llvm-ir-ls") {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        // 3. Fallback check for common user directories (~/.local/bin, ~/.cargo/bin)
        if let Ok(home) = std::env::var("HOME") {
            let local_bin = format!("{}/.local/bin/llvm-ir-ls", home);
            if fs::metadata(&local_bin).map_or(false, |stat| stat.is_file()) {
                self.cached_binary_path = Some(local_bin.clone());
                return Ok(local_bin);
            }
            let cargo_bin = format!("{}/.cargo/bin/llvm-ir-ls", home);
            if fs::metadata(&cargo_bin).map_or(false, |stat| stat.is_file()) {
                self.cached_binary_path = Some(cargo_bin.clone());
                return Ok(cargo_bin);
            }
        }

        // 4. Download latest binary from GitHub Releases
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (os, arch) = zed::current_platform();

        let binary_name = match os {
            zed::Os::Windows => "llvm-ir-ls.exe",
            _ => "llvm-ir-ls",
        };

        let version_dir = format!("llvm-ir-ls-{}", release.version);
        let binary_path = format!("{version_dir}/{binary_name}");

        if fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
            self.cached_binary_path = Some(binary_path.clone());
            return Ok(binary_path);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        let asset_suffix = match (os, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-unknown-linux-gnu.tar.gz",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-unknown-linux-gnu.tar.gz",
            (zed::Os::Mac, zed::Architecture::Aarch64) => "aarch64-apple-darwin.tar.gz",
            (zed::Os::Mac, zed::Architecture::X8664) => "x86_64-apple-darwin.tar.gz",
            (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-pc-windows-msvc.zip",
            _ => {
                let err = format!("unsupported OS or architecture: {os:?}, {arch:?}");
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(err.clone()),
                );
                return Err(err);
            }
        };

        let asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(asset_suffix))
            .ok_or_else(|| {
                format!(
                    "no binary release asset found ending with '{asset_suffix}' in release {}",
                    release.version
                )
            })?;

        let file_type = if asset_suffix.ends_with(".zip") {
            zed::DownloadedFileType::Zip
        } else {
            zed::DownloadedFileType::GzipTar
        };

        zed::download_file(&asset.download_url, &version_dir, file_type)
            .map_err(|e| format!("failed to download llvm-ir-ls release: {e}"))?;

        let _ = zed::make_file_executable(&binary_path);

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

zed::register_extension!(LlvmIrExtension);
