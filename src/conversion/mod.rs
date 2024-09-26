mod alignment;
mod element;

use crate::{
    error::{ConversionError, Error},
    parser::{Attribute, Value},
};

impl TryInto<f32> for Value {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let Value::Number(num) = self {
            Ok(num as f32)
        } else {
            Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to f32"
            ))))
        }
    }
}

impl TryInto<iced::Length> for Value {
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
            Value::Number(num) => Ok((num as f32).into()),
            _ => Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Unsupported {self:?}"
            )))),
        }
        /*
        if let Value::Number(num) = self {
            Ok((num as f32).into())
        } else {
            Err(Error::Conversion(ConversionError::InvalidType(format!(
                "Cannot convert {self:?} to iced::Length"
            ))))
        }
        */
    }
}

impl TryInto<iced::Padding> for Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::Padding, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<iced::Pixels> for Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        let val: f32 = self.try_into()?;
        Ok(val.into())
    }
}

impl TryInto<iced::Length> for Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::Length, Self::Error> {
        self.value.try_into()
    }
}

impl TryInto<iced::Pixels> for Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::Pixels, Self::Error> {
        self.value.try_into()
    }
}
