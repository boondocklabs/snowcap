use colored::{ColoredString, Colorize};

use std::{
    fmt::Write,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use thiserror::Error;

use super::{ParserContext, Rule};

#[derive(Error, Debug)]
pub struct ParseErrorContext {
    context: ParserContext,
    error: ParseError,
}

impl ParseErrorContext {
    pub fn new(context: ParserContext, error: ParseError) -> Self {
        Self { context, error }
    }
}

impl std::fmt::Display for ParseErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            ParseError::Pest(pest_error) => {
                write!(f, "{}", "\nMarkup parser error\n".red())?;
                write!(f, "{}", pest_error)
            }
            ParseError::Attribute(_e) => {
                self.display_error_context(f, "parsing Attributes".yellow())
            }
            ParseError::Gradient(_e) => self.display_error_context(f, "parsing Gradient".yellow()),
            ParseError::Color(_e) => self.display_error_context(f, "parsing Color".yellow()),
            _ => self.display_error_context(f, "processing markup".yellow()),
        }
    }
}

impl ParseErrorContext {
    fn display_error_context(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        msg: ColoredString,
    ) -> std::fmt::Result {
        let row = self.context.location.0;
        let column = self.context.location.1;

        // Get line from input
        let line = self
            .context
            .input
            .lines()
            .enumerate()
            .nth(row - 1)
            .unwrap_or((0, "<Line not found in input>"));

        let line = line.1;
        let remain = line.trim_start();

        let adjust = line.len() - remain.len();

        write!(
            f,
            "\n\n{} {} at line {}, column {}\n\n",
            "Error".red(),
            msg.red(),
            row.to_string().yellow(),
            column.to_string().yellow()
        )?;
        write!(f, "{}\n", remain.cyan())?;
        for _ in 0..column - adjust - 1 {
            f.write_char(' ')?;
        }
        write!(f, "{}", "^\n".bright_green())?;

        write!(f, "{}", self.error)
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),

    #[error("unsupported parse rule {0}")]
    UnsupportedRule(String),

    #[error("Unhandled {0}")]
    Unhandled(String),

    #[error("Missing {0}")]
    Missing(&'static str),

    #[error(transparent)]
    Color(#[from] pest::error::Error<super::color::Rule>),

    #[error(transparent)]
    Gradient(#[from] pest::error::Error<super::gradient::Rule>),

    #[error(transparent)]
    Attribute(#[from] pest::error::Error<super::attribute::Rule>),

    #[error(transparent)]
    Module(#[from] pest::error::Error<super::module::Rule>),

    #[error(transparent)]
    Value(#[from] pest::error::Error<super::value::Rule>),

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

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {

    use tracing::info;
    use tracing_test::traced_test;

    use crate::{Message, SnowcapParser};

    type M = Message<String>;

    #[traced_test]
    #[test]
    fn test_invalid_markup() {
        let res = SnowcapParser::<M>::parse_memory(
            r#"
            {




            {}
            "#,
        );
        assert!(res.is_err());
        let err = res.unwrap_err();
        info!("error: {}", err);
    }
}
