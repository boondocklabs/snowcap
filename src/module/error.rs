use thiserror::Error;

use super::ModuleHandleId;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("module '{0}' not found")]
    ModuleNotFound(String),

    #[error("module handle id {handle_id} not found: {msg}")]
    HandleNotFound {
        handle_id: ModuleHandleId,
        msg: String,
    },

    #[error("unknown module {0}")]
    Unknown(String),

    #[error("missing required argument {0}")]
    MissingArgument(String),

    #[error("invalid argument {0}")]
    InvalidArgument(String),

    #[error("io error {0}")]
    Io(#[from] std::io::Error),

    #[error("internal {0}")]
    Internal(Box<dyn std::error::Error + Send + Sync>),
}
