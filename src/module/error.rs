use thiserror::Error;

use super::HandleId;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("module handle id {handle_id} not found: {msg}")]
    HandleNotFound {
        handle_id: HandleId,
        msg: &'static str,
    },

    #[error("unknown module {0}")]
    Unknown(String),
}
