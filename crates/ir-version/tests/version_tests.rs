#[cfg(test)]
mod tests {
    use ir_version::*;

    #[test]
    fn test_version_rules() {
        let v14 = LlvmVersion::V14;
        let v18 = LlvmVersion::V18;

        assert!(!v14.supports_opaque_pointers());
        assert!(v14.supports_typed_pointers());

        assert!(v18.supports_opaque_pointers());
        assert!(v18.requires_opaque_pointers());
        assert!(!v18.supports_typed_pointers());

        assert!(v18.is_attribute_deprecated("readnone"));
        assert_eq!(
            v18.suggest_attribute_replacement("readnone"),
            Some("memory(none)")
        );
    }

    #[test]
    fn test_heuristic_detection() {
        let opaque_ir = "define void @f(ptr %p) {\n  store i32 0, ptr %p\n  ret void\n}";
        assert_eq!(detect_from_content(opaque_ir), LlvmVersion::default_stable());

        let typed_ir = "define void @f(i32* %p) {\n  store i32 0, i32* %p\n  ret void\n}";
        assert_eq!(detect_from_content(typed_ir), LlvmVersion::V14);
    }

    #[test]
    fn test_parse_llvm_version_output() {
        let s = "Debian LLVM version 18.1.8\n  Optimized build.";
        assert_eq!(parse_llvm_version_output(s), Some(18));
    }
}
