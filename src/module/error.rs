use thiserror::Error;

use super::HandleId;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("module handle id {handle_id} not found: {msg}")]
    ModuleNotFound {
        handle_id: HandleId,
        msg: &'static str,
    },
}
