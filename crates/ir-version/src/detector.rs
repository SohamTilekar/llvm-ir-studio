use crate::config::LlvmIrConfig;
use crate::version::LlvmVersion;
use std::path::Path;
use std::process::Command;

/// Version Resolution Order:
/// 1. Explicit `.llvm-ir-ls.toml` or LSP initialization setting (`llvmVersion = 18`).
/// 2. Auto-detect installed `llvm-as --version` / `opt --version` on `PATH`.
/// 3. Heuristic fallback from file content (e.g., presence of `ptr` vs typed pointers) defaulting to latest stable.
pub fn detect_version(
    file_path: Option<&Path>,
    content: &str,
    user_config_ver: Option<u32>,
) -> LlvmVersion {
    // 1. Explicit user config override
    if let Some(v) = user_config_ver {
        return LlvmVersion(v);
    }

    // 1b. Check `.llvm-ir-ls.toml`
    if let Some(path) = file_path {
        if let Some(cfg) = LlvmIrConfig::find_and_load(path) {
            if let Some(v) = cfg.version {
                return LlvmVersion(v);
            }
        }
    }

    // 2. Auto-detect installed llvm-as or opt on PATH
    if let Some(v) = detect_llvm_from_path() {
        return LlvmVersion(v);
    }

    // 3. Heuristic fallback from file content
    detect_from_content(content)
}

pub fn detect_llvm_from_path() -> Option<u32> {
    for bin in &["llvm-as", "opt"] {
        if let Ok(output) = Command::new(bin).arg("--version").output() {
            if output.status.success() {
                let out_str = String::from_utf8_lossy(&output.stdout);
                if let Some(v) = parse_llvm_version_output(&out_str) {
                    return Some(v);
                }
            }
        }
    }
    None
}

pub fn parse_llvm_version_output(output: &str) -> Option<u32> {
    // LLVM version output typically contains "LLVM version XX.Y.Z"
    for line in output.lines() {
        if let Some(idx) = line.find("LLVM version ") {
            let ver_part = &line[idx + "LLVM version ".len()..];
            let major_str = ver_part.split('.').next()?;
            if let Ok(major) = major_str.trim().parse::<u32>() {
                return Some(major);
            }
        }
    }
    None
}

/// Heuristic detection based on IR tokens:
/// - Presence of `ptr` without typed pointer syntax -> modern (LLVM 18+)
/// - Presence of `i32*`, `i8*`, `float*`, etc. -> legacy (LLVM 14)
pub fn detect_from_content(content: &str) -> LlvmVersion {
    let has_opaque_ptr = content.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|w| w == "ptr");

    let has_typed_ptr = content.lines().any(|line| {
        let trimmed = line.trim();
        // check for patterns like i8*, i32*, i64*, void*, float*, double*, %struct.*
        trimmed.contains("i8*")
            || trimmed.contains("i16*")
            || trimmed.contains("i32*")
            || trimmed.contains("i64*")
            || trimmed.contains("float*")
            || trimmed.contains("double*")
    });

    if has_typed_ptr && !has_opaque_ptr {
        LlvmVersion::V14
    } else {
        // Default to modern stable
        LlvmVersion::default_stable()
    }
}
