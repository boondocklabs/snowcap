use crate::{error::ConversionError, Error};

#[derive(Debug)]
pub enum DataType {
    Null,
    Image(iced::widget::image::Handle),
    QrCode(iced::widget::qr_code::Data),
}

impl<'a> TryInto<&'a iced::widget::qr_code::Data> for &'a DataType {
    type Error = Error;

    fn try_into(self) -> Result<&'a iced::widget::qr_code::Data, Self::Error> {
        if let DataType::QrCode(data) = self {
            return Ok(data);
        }
        Err(Error::Conversion(ConversionError::InvalidType(
            "Expecting DataType::QrCode".into(),
        )))
    }
}

#[derive(Debug)]
pub enum DataProviders {
    None,
    QrCode(QrDataWrapper),
}

impl<'a> TryInto<&'a iced::widget::qr_code::Data> for &'a DataProviders {
    type Error = Error;

    fn try_into(self) -> Result<&'a iced::widget::qr_code::Data, Self::Error> {
        match self {
            DataProviders::None => todo!(),
            DataProviders::QrCode(qr_data) => (&qr_data.0).try_into(),
        }
    }
}

#[derive(Debug)]
pub struct QrDataWrapper(DataType);

impl QrDataWrapper {
    pub fn new(data: iced::widget::qr_code::Data) -> Self {
        QrDataWrapper(DataType::QrCode(data))
    }

    pub fn data<'a>(&'a self) -> &'a iced::widget::qr_code::Data {
        if let DataType::QrCode(data) = &self.0 {
            return data;
        }
        panic!("no data")
    }
}
