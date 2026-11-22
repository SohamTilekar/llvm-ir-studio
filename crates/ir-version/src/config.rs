use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_FILE_NAME: &str = ".llvm-ir-ls.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlvmIrConfig {
    #[serde(rename = "llvm-version", alias = "llvmVersion", alias = "version")]
    pub version: Option<u32>,

    #[serde(rename = "verify-on-save", alias = "verifyOnSave")]
    pub verify_on_save: Option<bool>,

    #[serde(rename = "llvm-as-path", alias = "llvmAsPath")]
    pub llvm_as_path: Option<String>,

    #[serde(rename = "opt-path", alias = "optPath")]
    pub opt_path: Option<String>,
}

impl LlvmIrConfig {
    pub fn find_and_load(start_path: &Path) -> Option<Self> {
        let mut curr = if start_path.is_file() {
            start_path.parent()?
        } else {
            start_path
        };

        loop {
            let candidate = curr.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                if let Ok(content) = fs::read_to_string(&candidate) {
                    if let Ok(config) = toml::from_str::<LlvmIrConfig>(&content) {
                        return Some(config);
                    }
                }
            }
            curr = curr.parent()?;
        }
    }

    pub fn load_from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}
