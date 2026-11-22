pub mod config;
pub mod detector;
pub mod version;

pub use config::LlvmIrConfig;
pub use detector::{detect_from_content, detect_version, parse_llvm_version_output};
pub use version::LlvmVersion;
