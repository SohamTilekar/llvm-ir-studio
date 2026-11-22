use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LlvmVersion(pub u32);

impl LlvmVersion {
    pub const V14: LlvmVersion = LlvmVersion(14);
    pub const V15: LlvmVersion = LlvmVersion(15);
    pub const V16: LlvmVersion = LlvmVersion(16);
    pub const V17: LlvmVersion = LlvmVersion(17);
    pub const V18: LlvmVersion = LlvmVersion(18);
    pub const V19: LlvmVersion = LlvmVersion(19);
    pub const V20: LlvmVersion = LlvmVersion(20);
    pub const V21: LlvmVersion = LlvmVersion(21);

    pub fn default_stable() -> Self {
        Self::V18
    }

    pub fn major(&self) -> u32 {
        self.0
    }

    /// Opaque pointers (ptr) are supported in LLVM 15+
    pub fn supports_opaque_pointers(&self) -> bool {
        self.0 >= 15
    }

    /// Opaque pointers are mandatory in LLVM 17+ (typed pointers removed)
    pub fn requires_opaque_pointers(&self) -> bool {
        self.0 >= 17
    }

    /// Typed pointers (e.g. i32*, void*) were supported prior to LLVM 17
    pub fn supports_typed_pointers(&self) -> bool {
        self.0 < 17
    }

    /// In LLVM 16+, attributes like `readnone`, `readonly`, `writeonly`, `argmemonly`
    /// were replaced by `memory(...)` attributes.
    pub fn is_attribute_deprecated(&self, attr: &str) -> bool {
        if self.0 >= 16 {
            matches!(
                attr,
                "readnone"
                    | "readonly"
                    | "writeonly"
                    | "argmemonly"
                    | "inaccessiblememonly"
                    | "inaccessiblemem_or_argmemonly"
            )
        } else {
            false
        }
    }

    pub fn suggest_attribute_replacement(&self, attr: &str) -> Option<&'static str> {
        if self.0 >= 16 {
            match attr {
                "readnone" => Some("memory(none)"),
                "readonly" => Some("memory(read)"),
                "writeonly" => Some("memory(write)"),
                "argmemonly" => Some("memory(argmem: readwrite)"),
                "inaccessiblememonly" => Some("memory(inaccessiblemem: readwrite)"),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl fmt::Display for LlvmVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LLVM {}", self.0)
    }
}
