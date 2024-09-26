use thiserror::Error;

use crate::parser::Rule;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unsupported parse rule {0}")]
    UnsupportedRule(String),
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("invalid type {0}")]
    InvalidType(String),
    #[error("unsupported attribute {0}")]
    UnsupportedAttribute(String),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Conversion(#[from] ConversionError),

    #[error("Required attribute {0} missing")]
    MissingAttribute(String),

    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
}
