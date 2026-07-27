use thiserror::Error;

#[derive(Error, Debug)]
pub enum FfiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Preprocessing error: {0}")]
    Preprocess(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unsupported C construct: {0}")]
    Unsupported(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("Header not found: {0}")]
    HeaderNotFound(String),
}

pub type FfiResult<T> = Result<T, FfiError>;
