use crate::{attribute::AttributeKind, ConversionError};

use super::{error::ParseError, ParserContext};
use iced::widget::text::IntoFragment;
use pest::{iterators::Pair, Parser as _};
use pest_derive::Parser;
use std::{borrow::Borrow, fmt::Write, ops::Deref};
use strum::EnumDiscriminants;
use tracing::debug;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Value {
    context: Option<ParserContext>,
    inner: ValueData,
}

impl Value {
    pub fn new(inner: ValueData) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    pub fn new_string(val: String) -> Self {
        Self {
            inner: ValueData::String(val),
            context: None,
        }
    }

    pub fn new_float(val: f64) -> Self {
        Self {
            inner: ValueData::Float(val),
            context: None,
        }
    }

    pub fn new_integer(val: u64) -> Self {
        Self {
            inner: ValueData::Integer(val),
            context: None,
        }
    }

    pub fn new_bool(val: bool) -> Self {
        Self {
            inner: ValueData::Boolean(val),
            context: None,
        }
    }

    pub fn new_array(val: Vec<Self>) -> Self {
        Self {
            inner: ValueData::Array(val),
            context: None,
        }
    }

    pub fn new_attribute_kind(val: AttributeKind) -> Self {
        Self {
            inner: ValueData::AttributeKind(val),
            context: None,
        }
    }

    pub fn with_context(mut self, context: ParserContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn inner(&self) -> &ValueData {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut ValueData {
        &mut self.inner
    }

    /// Return true if this Value inner data kind is [`ValueDataKind`]
    pub fn is_kind(&self, kind: ValueDataKind) -> bool {
        kind == ValueDataKind::from(self.inner())
    }

    /// Get a f64 from this Value. If the inner type is an integer,
    /// it will be converted to float and returned.
    pub fn float(&self) -> Result<f64, ConversionError> {
        match self.inner() {
            ValueData::Float(float) => Ok(*float),
            ValueData::Integer(integer) => Ok(*integer as f64),
            _ => Err(ConversionError::InvalidType(format!(
                "expecting ValueKind::Float. Got {:?}",
                self.inner()
            ))),
        }
    }

    /// Get a u64 from this Value. Inner ValueData kind must be Integer
    pub fn integer(&self) -> Result<u64, ConversionError> {
        match self.inner() {
            ValueData::Integer(integer) => Ok(*integer),
            _ => Err(ConversionError::InvalidType(format!(
                "expecting ValueKind::Integer. Got {:?}",
                self.inner()
            ))),
        }
    }

    /// Get a bool from this Value. Inner ValueData kind must be Boolean
    pub fn boolean(&self) -> Result<bool, ConversionError> {
        match self.inner() {
            ValueData::Boolean(boolean) => Ok(*boolean),
            _ => Err(ConversionError::InvalidType(format!(
                "expecting ValueKind::Boolean. Got {:?}",
                self.inner()
            ))),
        }
    }

    pub fn array(&self) -> Result<&Vec<Value>, ConversionError> {
        if let ValueData::Array(array) = self.inner() {
            Ok(array)
        } else {
            Err(ConversionError::InvalidType(
                "expecting ValueKind::Array".into(),
            ))
        }
    }
}

impl Deref for Value {
    type Target = ValueData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(ValueDataKind))]
#[derive(Default, Clone, Debug)]
pub enum ValueData {
    #[default]
    None,
    String(String),
    Float(f64),
    Integer(u64),
    Boolean(bool),
    Array(Vec<Value>),
    AttributeKind(AttributeKind),
}

impl Eq for ValueData {}

impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    (a - b).abs() < f64::EPSILON
                }
            }
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for ValueData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueData::String(s) => f.write_str(s),
            ValueData::Float(num) => f.write_fmt(format_args!("{}f64", num)),
            ValueData::Integer(num) => f.write_fmt(format_args!("{}u64", num)),
            ValueData::Boolean(b) => f.write_fmt(format_args!("{}", b)),
            ValueData::Array(vec) => {
                f.write_char('[')?;
                let mut iter = vec.iter().peekable();
                while let Some(val) = iter.next() {
                    write!(f, "{}", val)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                f.write_char(']')
            }
            ValueData::AttributeKind(kind) => f.write_fmt(format_args!("{:?}", kind)),
            ValueData::None => write!(f, "None"),
        }
    }
}

impl Borrow<String> for Value {
    fn borrow(&self) -> &String {
        match &self.inner {
            ValueData::String(s) => s,
            _ => panic!("Cannot borrow string for non-string typed value"),
        }
    }
}

impl Borrow<str> for Value {
    fn borrow(&self) -> &str {
        match &self.inner {
            ValueData::String(s) => s,
            _ => panic!("Cannot borrow string for non-string typed value"),
        }
    }
}

impl<'a> Into<&'a String> for &'a Value {
    fn into(self) -> &'a String {
        match &self.inner {
            ValueData::String(s) => s,
            _ => panic!("Cannot borrow string for non-string typed value"),
        }
    }
}

impl<'a> IntoFragment<'a> for &ValueData {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        self.into()
    }
}

/// Converts a [`Value`] reference into a [`Cow<'a, str>`].
///
/// This conversion is used to represent different `Value` variants as string data.
///
/// The possible variants and their conversions:
/// - `Value::String(s)` returns the cloned string.
/// - `Value::Number(n)` is formatted as a string.
/// - `Value::Boolean(b)` is formatted as a string.
/// - `Value::Null` is formatted as `"null"`.
/// - `Value::DataSource`:
///   - If the `DataProvider` is a `File` and its data is `Text`, returns the text data.
///   - Otherwise, it returns `"Unsupported DataProvider"` or `"Unknown DataType"`.
///
impl<'a> Into<std::borrow::Cow<'a, str>> for &ValueData {
    fn into(self) -> std::borrow::Cow<'a, str> {
        match self {
            ValueData::String(s) => s.clone().into(),
            ValueData::Float(n) => format!("{n}").into(),
            ValueData::Integer(n) => format!("{n}").into(),
            ValueData::Boolean(b) => format!("{b}").into(),
            ValueData::Array(_value) => todo!(),
            ValueData::AttributeKind(_kind) => todo!(),
            ValueData::None => format!("None").into(),
        }
    }
}

#[derive(Parser)]
#[grammar = "parser/value.pest"]
pub struct ValueParser;

impl ValueParser {
    fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
        let mut value = Value::default();

        for pair in pair.into_inner() {
            value = match pair.as_rule() {
                Rule::string => Value::new_string(pair.into_inner().as_str().into()),
                Rule::float => Value::new_float(pair.as_str().parse().map_err(ParseError::Float)?),
                Rule::integer => {
                    Value::new_integer(pair.as_str().parse().map_err(ParseError::Integer)?)
                }
                Rule::boolean => {
                    Value::new_bool(pair.as_str().parse().map_err(ParseError::Boolean)?)
                }
                Rule::none => Value::default(),

                Rule::array => {
                    let mut values = Vec::new();
                    for pair in pair.into_inner() {
                        values.push(Self::parse_value(pair)?)
                    }
                    Value::new_array(values)
                }

                // Return the module when the EOI rule is emitted

                // Handle unsupported rules
                _ => {
                    return Err(ParseError::UnsupportedRule(format!(
                        "{}: {} {:?}",
                        file!(),
                        line!(),
                        pair.as_rule()
                    )));
                }
            };
        }

        Ok(value)
    }

    pub fn parse_str(data: &str, context: &ParserContext) -> Result<Value, ParseError> {
        debug!("Parsing value {data}");

        let pairs = Self::parse(Rule::value, data)?;

        let mut value = Value::default();

        if let Some(root) = pairs.into_iter().last() {
            for pair in root.into_inner() {
                match pair.as_rule() {
                    Rule::values => value = Self::parse_value(pair)?,
                    Rule::EOI => return Ok(value.with_context(context.clone())),
                    _ => {
                        return Err(ParseError::UnsupportedRule(format!(
                            "{}: {} {:?}",
                            file!(),
                            line!(),
                            pair.as_rule()
                        )));
                    }
                }
            }
        }

        // Ok path is early return inside the loop above on EOI event,
        // if the loop returned, EOI was not emitted.
        Err(ParseError::Missing("EOI not emitted"))
    }
}

#[cfg(test)]
mod tests {
    use super::ValueParser;
    use crate::parser::{value::ValueDataKind, ParserContext};
    use approx::abs_diff_eq;

    #[test]
    fn string() {
        let value = ValueParser::parse_str(r#""foo""#, &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::String));
        assert_eq!(value.to_string(), "foo");

        // Check true equality
        let other_value = ValueParser::parse_str(r#""foo""#, &ParserContext::default()).unwrap();
        assert!(value == other_value);

        // Check false equality
        let other_value = ValueParser::parse_str(r#""bar""#, &ParserContext::default()).unwrap();
        assert!(value != other_value);
    }

    #[test]
    fn float() {
        let value = ValueParser::parse_str("3.14", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Float));
        let float = value.float();
        assert!(float.is_ok());
        assert!(abs_diff_eq!(float.unwrap(), 3.14));

        // Check true equality
        let other_value = ValueParser::parse_str("3.14", &ParserContext::default()).unwrap();
        assert!(value == other_value);

        // Check false equality
        let other_value = ValueParser::parse_str("3.5", &ParserContext::default()).unwrap();
        assert!(value != other_value);

        // Check negative
        let value = ValueParser::parse_str("-3.14", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Float));
        let float = value.float();
        assert!(float.is_ok());
        assert!(abs_diff_eq!(float.unwrap(), -3.14));

        let value = ValueParser::parse_str("3.", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Float));
        let float = value.float();
        assert!(float.is_ok());
        assert!(abs_diff_eq!(float.unwrap(), 3.0));

        // Should be able to get a float from an integer type
        let value = ValueParser::parse_str("3", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Integer));
        let float = value.float();
        assert!(float.is_ok());
        assert!(abs_diff_eq!(float.unwrap(), 3.0));
    }

    #[test]
    fn integer() {
        let value = ValueParser::parse_str("3", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Integer));

        let integer = value.integer();
        assert!(integer.is_ok());
        assert!(integer.unwrap() == 3);

        // Check true equality
        let other_value = ValueParser::parse_str("3", &ParserContext::default()).unwrap();
        assert!(value == other_value);

        // Check false equality
        let other_value = ValueParser::parse_str("4", &ParserContext::default()).unwrap();
        assert!(value != other_value);

        let value = ValueParser::parse_str("3.0", &ParserContext::default()).unwrap();
        let integer = value.integer();
        assert!(integer.is_err());
    }

    #[test]
    fn boolean() {
        let value_true = ValueParser::parse_str("true", &ParserContext::default()).unwrap();
        assert!(value_true.is_kind(ValueDataKind::Boolean));
        let boolean = value_true.boolean();
        assert!(boolean.is_ok());
        assert!(boolean.unwrap() == true);

        let value_false = ValueParser::parse_str("false", &ParserContext::default()).unwrap();
        assert!(value_false.is_kind(ValueDataKind::Boolean));
        let boolean = value_false.boolean();
        assert!(boolean.is_ok());
        assert!(boolean.unwrap() == false);

        // Check equality
        assert!(value_true != value_false);

        let other_true = ValueParser::parse_str("true", &ParserContext::default()).unwrap();
        assert!(value_true == other_true);
    }

    #[test]
    fn array() {
        let value = ValueParser::parse_str("[1,2,3]", &ParserContext::default()).unwrap();
        assert!(value.is_kind(ValueDataKind::Array));

        let array = value.array();
        assert!(array.is_ok());

        let array = array.unwrap();
        assert_eq!(array.len(), 3);

        assert_eq!(array[0].integer().unwrap(), 1);
        assert_eq!(array[1].integer().unwrap(), 2);
        assert_eq!(array[2].integer().unwrap(), 3);
    }
}
