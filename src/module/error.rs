use thiserror::Error;

use super::HandleId;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("module {0} not found")]
    ModuleNotFound(String),

    #[error("module handle id {handle_id} not found: {msg}")]
    HandleNotFound { handle_id: HandleId, msg: String },

    #[error("unknown module {0}")]
    Unknown(String),

    #[error("missing required argument {0}")]
    MissingArgument(String),

    #[error("invalid argument {0}")]
    InvalidArgument(String),
}
