use thiserror::Error;

pub type Result<T> = std::result::Result<T, MiningError>;

#[derive(Error, Debug)]
pub enum MiningError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Mining failed: {0}")]
    MiningFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Data load error: {0}")]
    DataLoadError(String),
}
