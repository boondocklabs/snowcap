use std::{
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unsupported parse rule {0}")]
    UnsupportedRule(String),

    #[error("Unhandled {0}")]
    Unhandled(String),

    #[error(transparent)]
    Color(#[from] pest::error::Error<super::color::Rule>),

    #[error(transparent)]
    Gradient(#[from] pest::error::Error<super::gradient::Rule>),

    #[error(transparent)]
    Attribute(#[from] pest::error::Error<super::attribute::Rule>),

    #[error("Invalid Color {0}")]
    InvalidColor(String),

    #[error(transparent)]
    Float(ParseFloatError),

    #[error(transparent)]
    Integer(ParseIntError),

    #[error(transparent)]
    Boolean(ParseBoolError),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    QrCode(#[from] iced::widget::qr_code::Error),

    #[error(transparent)]
    Borrow(#[from] std::cell::BorrowMutError),
}
