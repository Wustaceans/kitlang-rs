use thiserror::Error;

/// Errors that can occur during compiler detection and invocation.
#[derive(Error, Debug)]
pub enum Error {
    /// A compilation error with a descriptive message.
    #[error("Compilation error: {0}")]
    CompileError(String),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
