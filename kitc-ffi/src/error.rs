use thiserror::Error;

/// Errors that can occur during C header processing (preprocessing, parsing, type conversion).
#[derive(Error, Debug)]
pub enum FfiError {
    /// An I/O error occurred (e.g., reading a header file).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Preprocessing failed (e.g., include resolution, macro expansion).
    #[error("Preprocessing error: {0}")]
    Preprocess(String),

    /// Parsing failed (tree-sitter-c could not parse the preprocessed source).
    #[error("Parse error: {0}")]
    Parse(String),

    /// Encountered a C construct that is not supported by the parser or type mapper.
    #[error("Unsupported C construct: {0}")]
    Unsupported(String),

    /// Failed to convert a C type to Kit's internal type representation.
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// The specified header file was not found on the filesystem.
    #[error("Header not found: {0}")]
    HeaderNotFound(String),
}

/// Result type alias for FFI operations.
pub type FfiResult<T> = Result<T, FfiError>;
