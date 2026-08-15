/// Test that kitc-ffi can parse a realistic C header and extract function prototypes.
#[cfg(test)]
mod tests {
    use kitc_ffi::PreprocessConfig;
    use kitc_ffi::extract_from_preprocessed;
    use kitc_ffi::extract_header_from_source;
    use kitc_ffi::types::*;

    const TEST_HEADER: &str = include_str!("test_header.h");

    #[test]
    fn test_parse_function_prototypes() {
        let config = PreprocessConfig::new().with_builtin_headers(true);
        let decls =
            extract_header_from_source(TEST_HEADER, &config).expect("Failed to parse test header");

        // Check function count
        let func_names: Vec<&str> = decls.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            func_names.contains(&"add"),
            "Expected 'add' function, got: {:?}",
            func_names
        );
        assert!(
            func_names.contains(&"printf"),
            "Expected 'printf' function, got: {:?}",
            func_names
        );
        assert!(
            func_names.contains(&"distance"),
            "Expected 'distance' function, got: {:?}",
            func_names
        );

        // Check specific function signatures
        let add = decls
            .functions
            .iter()
            .find(|f| f.name == "add")
            .expect("add function not found");
        assert_eq!(add.return_type, CType::Int);
        assert_eq!(add.params.len(), 2);
        assert_eq!(add.params[0].ty, CType::Int);
        assert_eq!(add.params[1].ty, CType::Int);

        // Check variadic function
        let printf = decls
            .functions
            .iter()
            .find(|f| f.name == "printf")
            .expect("printf function not found");
        assert!(printf.is_variadic);
    }

    #[test]
    fn test_parse_structs() {
        let config = PreprocessConfig::new().with_builtin_headers(true);
        let decls =
            extract_header_from_source(TEST_HEADER, &config).expect("Failed to parse test header");

        // Check that typedefs are extracted
        let typedef_names: Vec<&str> = decls.typedefs.iter().map(|t| t.name.as_str()).collect();
        assert!(
            typedef_names.contains(&"ulong"),
            "Expected 'ulong' typedef, got: {:?}",
            typedef_names
        );
    }

    #[test]
    fn test_parse_enums() {
        let config = PreprocessConfig::new().with_builtin_headers(true);
        let decls =
            extract_header_from_source(TEST_HEADER, &config).expect("Failed to parse test header");

        // Check global variables
        let global_names: Vec<&str> = decls.globals.iter().map(|g| g.name.as_str()).collect();
        assert!(
            global_names.contains(&"global_counter"),
            "Expected 'global_counter' global, got: {:?}",
            global_names
        );
        assert!(
            global_names.contains(&"version_string"),
            "Expected 'version_string' global, got: {:?}",
            global_names
        );
    }

    #[test]
    fn test_parse_from_preprocessed() {
        // Simulate preprocessed C (no includes, macros expanded)
        let preprocessed = r#"
            typedef unsigned long size_t;
            int add(int a, int b);
            void *malloc(size_t size);
        "#;

        let decls =
            extract_from_preprocessed(preprocessed).expect("Failed to parse preprocessed C");

        assert_eq!(decls.functions.len(), 2);
        assert_eq!(decls.functions[0].name, "add");
        assert_eq!(decls.functions[1].name, "malloc");
    }

    #[test]
    fn test_empty_header() {
        let decls = extract_from_preprocessed("/* just a comment */")
            .expect("Failed to parse empty header");
        assert!(decls.is_empty());
    }

    #[test]
    fn test_header_with_error_records_skipped_nodes() {
        // A syntax error inside the header: valid siblings are still extracted, and the
        // broken nodes are recorded so callers can diagnose partial results.
        let source = r#"
            int valid_function(int x);
            this is not valid c
            void another_function(void);
        "#;
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(
            decls.functions.iter().map(|f| &f.name).collect::<Vec<_>>(),
            vec!["valid_function", "another_function"]
        );
        assert!(
            !decls.skipped_nodes.is_empty(),
            "broken nodes must be recorded"
        );
        assert!(
            decls
                .skipped_nodes
                .iter()
                .any(|s| s.line > 0 && s.column > 0)
        );
    }

    #[test]
    fn test_whole_file_error_keeps_valid_siblings() {
        // Recovery can collapse the whole file into a single root ERROR node whose direct
        // children report no error; valid declarations inside it are still extracted, and
        // the parse does not fail outright.
        let source =
            "int valid_function(int x);\nint broken_function( {\nvoid another_function(void);\n";
        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(
            decls.functions.iter().map(|f| &f.name).collect::<Vec<_>>(),
            vec!["valid_function", "another_function"]
        );
    }

    #[test]
    fn test_pointer_return_type() {
        let source = r#"
            void *create_buffer(int size);
        "#;
        let decls = extract_from_preprocessed(source).unwrap();
        let func = &decls.functions[0];
        assert_eq!(func.name, "create_buffer");
        match &func.return_type {
            CType::Ptr(inner, _) => assert_eq!(**inner, CType::Void),
            other => panic!("Expected pointer to void, got {other}"),
        }
    }

    #[test]
    fn test_const_char_ptr_param() {
        let source = r#"
            size_t strlen(const char *s);
        "#;
        let config = PreprocessConfig::new().with_builtin_headers(false);
        let decls = extract_header_from_source(source, &config).unwrap();
        let func = &decls.functions[0];
        assert_eq!(func.name, "strlen");
        assert_eq!(func.params.len(), 1);
        match &func.params[0].ty {
            CType::Ptr(inner, qualifiers) => {
                assert_eq!(**inner, CType::Char);
                assert!(qualifiers.contains(&CQualifier::Const));
            }
            other => panic!("Expected const char pointer, got {other}"),
        }
    }

    #[test]
    fn test_typedef_extraction() {
        let source = r#"
            typedef unsigned long long uint64_t;
            typedef int myint;
        "#;
        let config = PreprocessConfig::new().with_builtin_headers(false);
        let decls = extract_header_from_source(source, &config).unwrap();
        assert_eq!(decls.typedefs.len(), 2);
        assert_eq!(decls.typedefs[0].name, "uint64_t");
        assert_eq!(decls.typedefs[1].name, "myint");
    }

    #[test]
    fn test_anonymous_struct_typedef_uses_alias_name() {
        let source = r#"
            typedef struct {
                int quot;
                int rem;
            } div_t;
            div_t div(int numer, int denom);
        "#;
        let config = PreprocessConfig::new().with_builtin_headers(false);
        let decls = extract_header_from_source(source, &config).unwrap();

        assert_eq!(decls.structs.len(), 1);
        assert_eq!(decls.structs[0].name, "div_t");
        assert_eq!(decls.structs[0].fields[0].name.as_deref(), Some("quot"));
        assert_eq!(decls.structs[0].fields[1].name.as_deref(), Some("rem"));
        assert_eq!(
            decls.functions[0].return_type,
            CType::Named("div_t".to_string())
        );
    }

    #[test]
    fn test_macos_div_prototype_with_pure_attribute() {
        let source = r#"
            typedef struct {
                int quot;
                int rem;
            } div_t;
            div_t div(int, int) __attribute__((__pure__));
        "#;

        let decls = extract_from_preprocessed(source).unwrap();
        assert_eq!(decls.structs.len(), 1);
        assert_eq!(decls.functions.len(), 1);
        assert_eq!(decls.functions[0].name, "div");
        assert_eq!(
            decls.functions[0].return_type,
            CType::Named("div_t".to_string())
        );
    }
}
