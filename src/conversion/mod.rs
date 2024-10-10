mod alignment;
mod column;
mod container;
mod data;
mod element;
pub(crate) mod node;
mod row;
mod stack;
mod text;
pub(crate) mod theme;
pub(crate) mod widget;

use crate::{attribute::Attribute, error::ConversionError, parser::Value};

/// Implements `TryInto` to convert a reference to `Value` into a reference to `String`.
///
/// # Type Parameters
///
/// * `'a` - A lifetime parameter.
///
/// # Errors
///
/// Returns a `ConversionError::InvalidType` if the value is not of type `String`.
///
/// # Examples
///
/// ```
/// use snowcap::Value;
/// use snowcap::ConversionError;
/// let value = Value::String("example".to_string());
/// let string_ref: Result<&String, ConversionError> = (&value).try_into();
/// assert_eq!(string_ref.unwrap(), "example");
/// ```
///
/// ```
/// use snowcap::Value;
/// use snowcap::ConversionError;
/// let value = Value::Number(42.into());
/// let result: Result<&String, ConversionError> = (&value).try_into();
/// assert!(result.is_err());
impl<'a> TryInto<&'a String> for &'a Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a String, Self::Error> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to f32"
            )))
        }
    }
}

/// Implements `TryInto<f32>` for a reference to `Value`.
///
/// This implementation allows attempting to convert a `&Value` reference into an `f32` value.
///
/// # Errors
///
/// If the `Value` is not of the type `Number`, the method returns an `Error::Conversion` containing a
/// `ConversionError::InvalidType` indicating that the conversion failed.
///
/// # Examples
///
/// ```
/// use snowcap::Value;
/// use snowcap::ConversionError;
/// let value = Value::Number(42.into());
/// let float_result: Result<f32, ConversionError> = (&value).try_into();
/// assert_eq!(float_result.unwrap(), 42.0);
/// ```
///
/// ```
/// use snowcap::Value;
/// use snowcap::ConversionError;
/// let value = Value::String("not a number".to_string());
/// let float_result: Result<f32, ConversionError> = (&value).try_into();
/// assert!(float_result.is_err());
/// ```
impl TryInto<f32> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let Value::Number(num) = self {
            Ok(*num as f32)
        } else {
            Err(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to f32"
            )))
        }
    }
}

impl TryInto<u16> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<u16, Self::Error> {
        if let Value::Number(num) = self {
            Ok(*num as u16)
        } else {
            Err(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to u16"
            )))
        }
    }
}

impl TryInto<bool> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Value::Boolean(b) = self {
            Ok(*b)
        } else {
            Err(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to bool"
            )))
        }
    }
}

impl TryInto<iced::Length> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        match self {
            Value::String(str) => match str.as_str() {
                "fill" => Ok(iced::Length::Fill),
                "shrink" => Ok(iced::Length::Shrink),
                _ => Err(ConversionError::InvalidType(format!(
                    "Expecting fill or shrink"
                ))),
            },
            Value::Number(num) => Ok((*num as f32).into()),
            _ => Err(ConversionError::InvalidType(format!(
                "Unsupported {self:?}"
            ))),
        }
    }
}

impl TryInto<iced::Padding> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Padding, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<iced::Pixels> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<String> for &Value {
    type Error = ConversionError;

    fn try_into(self) -> Result<String, Self::Error> {
        if let Value::String(s) = self {
            Ok(s.clone())
        } else {
            Err(ConversionError::InvalidType(
                "Expecting Value::String".into(),
            ))
        }
    }
}

impl TryInto<iced::Length> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<iced::Length> for Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<iced::Pixels> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<iced::Pixels> for Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<bool> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<bool, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<bool> for Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<bool, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<u16> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<u16, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<u16> for Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<u16, Self::Error> {
        (&*self.value()).try_into()
    }
}

/*
impl<'a> TryInto<&'a String> for &'a Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a String, Self::Error> {
        (&*self.value()).try_into()
    }
}
*/
