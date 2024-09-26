use crate::{
    error::{ConversionError, Error},
    parser::{Attribute, Value},
};

impl TryInto<iced::alignment::Vertical> for Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::alignment::Vertical, Self::Error> {
        match self {
            Value::String(str) => match str.as_str() {
                "top" => Ok(iced::alignment::Vertical::Top),
                "center" => Ok(iced::alignment::Vertical::Center),
                "bottom" => Ok(iced::alignment::Vertical::Bottom),
                _ => Err(Error::Conversion(ConversionError::InvalidType(
                    "Expecting top, center, or bottom for Vertical alignment".into(),
                ))),
            },
            Value::Number(_) => todo!(),
            Value::Boolean(_) => todo!(),
            Value::Null => todo!(),
        }
    }
}

impl TryInto<iced::alignment::Vertical> for Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::alignment::Vertical, Self::Error> {
        self.value.try_into()
    }
}

impl TryInto<iced::alignment::Horizontal> for Value {
    type Error = Error;

    fn try_into(self) -> Result<iced::alignment::Horizontal, Self::Error> {
        match self {
            Value::String(str) => match str.as_str() {
                "left" => Ok(iced::alignment::Horizontal::Left),
                "center" => Ok(iced::alignment::Horizontal::Center),
                "right" => Ok(iced::alignment::Horizontal::Right),
                _ => Err(Error::Conversion(ConversionError::InvalidType(
                    "Expecting left, center, or right for Horizontal alignment".into(),
                ))),
            },
            Value::Number(_) => todo!(),
            Value::Boolean(_) => todo!(),
            Value::Null => todo!(),
        }
    }
}

impl TryInto<iced::alignment::Horizontal> for Attribute {
    type Error = Error;

    fn try_into(self) -> Result<iced::alignment::Horizontal, Self::Error> {
        self.value.try_into()
    }
}
