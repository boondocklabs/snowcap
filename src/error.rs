use std::cell::BorrowMutError;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::attribute::Attribute;
use crate::parser::error::ParseError;
use crate::parser::Rule;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("invalid type {0}")]
    InvalidType(String),

    #[error("{0:?} for {1:?}")]
    UnsupportedAttribute(Attribute, String),

    #[error("unsupported widget {0}")]
    UnsupportedWidget(String),

    #[error("missing {0}")]
    Missing(String),

    #[error("unknown {0}")]
    Unknown(String),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error("downcast {0}")]
    Downcast(String),

    #[error(transparent)]
    BorrowMut(#[from] BorrowMutError),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Conversion(#[from] ConversionError),

    #[error("Node does not have an ID")]
    MissingId,

    #[error("Required attribute {0} missing")]
    MissingAttribute(String),

    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error("Unhandled {0}")]
    Unhandled(String),

    #[error(transparent)]
    Encoding(FromUtf8Error),

    #[error("Node {0} Not Found")]
    NodeNotFound(arbutus::NodeId),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Tokio(tokio::task::JoinError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Notify(#[from] notify::Error),

    #[error(transparent)]
    BorrowMut(#[from] BorrowMutError),

    #[error("Deadlock {0}")]
    Deadlock(String),
}
