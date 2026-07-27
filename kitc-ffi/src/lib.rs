pub mod error;
pub mod parse;
pub mod preprocess;
pub mod system_headers;
pub mod types;

use std::path::Path;

pub use error::{FfiError, FfiResult};
pub use preprocess::{PreprocessConfig, Target};
pub use types::*;

/// Extract all C declarations from a header file.
///
/// Preprocesses the header with includium, then parses with tree-sitter-c.
/// The includium preprocessor resolves `#include` directives using the
/// configured include resolver, which may use fake system headers.
pub fn extract_header(header_path: &str, config: &PreprocessConfig) -> FfiResult<CDeclarations> {
    let path = Path::new(header_path);
    if !path.exists() {
        return Err(FfiError::HeaderNotFound(header_path.to_string()));
    }

    let preprocessed = preprocess::preprocess_header(path, config)?;
    parse::parse_c_header(&preprocessed)
}

/// Extract declarations from a header source string.
///
/// Preprocesses the source through includium first, then parses.
/// Use this when the header source is already in memory.
/// For already-preprocessed source, use `extract_from_preprocessed`.
pub fn extract_header_from_source(
    source: &str,
    config: &PreprocessConfig,
) -> FfiResult<CDeclarations> {
    let preprocessed = preprocess::preprocess_source_from_string(source, config)?;
    parse::parse_c_header(&preprocessed)
}

/// Extract declarations from preprocessed C source (no preprocessing step).
///
/// Useful when the caller has already preprocessed the header,
/// or when parsing test strings that contain no preprocessor directives.
pub fn extract_from_preprocessed(source: &str) -> FfiResult<CDeclarations> {
    parse::parse_c_header(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_function() {
        let source = "int add(int a, int b);";
        let config = PreprocessConfig::new().with_fake_system_headers(false);
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "add");
        assert_eq!(decls.functions[0].params.len(), 2);
        assert_eq!(decls.functions[0].return_type, CType::Int);
    }

    #[test]
    fn test_extract_void_function() {
        let source = "void greet(const char *name);";
        let config = PreprocessConfig::new().with_fake_system_headers(false);
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "greet");
        assert_eq!(decls.functions[0].return_type, CType::Void);
        assert_eq!(decls.functions[0].params.len(), 1);
    }

    #[test]
    fn test_extract_variadic_function() {
        let source = "int printf(const char *format, ...);";
        let config = PreprocessConfig::new().with_fake_system_headers(false);
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "printf");
        assert!(decls.functions[0].is_variadic);
    }

    #[test]
    fn test_extract_typedef() {
        let source = "typedef unsigned long size_t;";
        let config = PreprocessConfig::new().with_fake_system_headers(false);
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.typedefs.len(), 1);
        assert_eq!(decls.typedefs[0].name, "size_t");
    }

    #[test]
    fn test_extract_empty_header() {
        let source = "/* just a comment */";
        let decls = extract_from_preprocessed(source).unwrap();
        assert!(decls.is_empty());
    }

    #[test]
    fn test_pointer_return_type() {
        let source = "void *malloc(size_t size);";
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "malloc");
        match &decls.functions[0].return_type {
            CType::Ptr(inner, _) => assert_eq!(**inner, CType::Void),
            other => panic!("Expected pointer to void, got {other}"),
        }
    }

    #[test]
    fn test_const_char_ptr() {
        let source = "const char *greeting(void);";
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "greeting");
        match &decls.functions[0].return_type {
            CType::Ptr(inner, qualifiers) => {
                assert_eq!(**inner, CType::Char);
                assert!(qualifiers.contains(&CQualifier::Const));
            }
            other => panic!("Expected const char pointer, got {other}"),
        }
    }
}
