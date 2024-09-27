mod alignment;
mod column;
mod container;
mod element;
mod row;
mod stack;
mod text;
mod widget;

use crate::{
    error::{ConversionError, Error},
    parser::{Attribute, Value},
};

impl TryInto<f32> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let Value::Number(num) = self {
            Ok(*num as f32)
        } else {
            Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to f32"
            ))))
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
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Value::Boolean(b) = self {
            Ok(*b)
        } else {
            Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to bool"
            ))))
        }
    }
}

impl TryInto<iced::Length> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        match self {
            Value::String(str) => match str.as_str() {
                "fill" => Ok(iced::Length::Fill),
                "shrink" => Ok(iced::Length::Shrink),
                _ => Err(Error::Conversion(ConversionError::InvalidType(format!(
                    "Expecting fill or shrink"
                )))),
            },
            Value::Number(num) => Ok((*num as f32).into()),
            _ => Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Unsupported {self:?}"
            )))),
        }
    }
}

impl TryInto<iced::Padding> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::Padding, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<iced::Pixels> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<iced::Length> for &Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        (&self.value).try_into()
    }
}

impl TryInto<iced::Pixels> for &Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        (&self.value).try_into()
    }
}

impl TryInto<bool> for &Attribute {
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        (&self.value).try_into()
    }
}

impl TryInto<u16> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<u16, Self::Error> {
        (&self.value).try_into()
    }
}
