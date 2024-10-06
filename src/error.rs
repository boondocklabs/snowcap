use std::string::FromUtf8Error;

use thiserror::Error;

use crate::parser::error::ParseError;
use crate::parser::{NodeId, Rule};

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("invalid type {0}")]
    InvalidType(String),

    #[error("unsupported attribute {0}")]
    UnsupportedAttribute(String),

    #[error("missing {0}")]
    Missing(String),

    #[error("unknown {0}")]
    Unknown(String),

    #[error(transparent)]
    Parse(#[from] ParseError),
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
    Notify(#[from] notify::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error("Unhandled {0}")]
    Unhandled(String),

    #[error(transparent)]
    Tokio(tokio::task::JoinError),

    #[error(transparent)]
    Encoding(FromUtf8Error),

    #[error("Node {0} Not Found")]
    NodeNotFound(NodeId),
}
