use std::{fs::File, io::Read, path::PathBuf};

use iced::widget::markdown::Item;
use tracing::info;

use crate::{error::ConversionError, Error};

#[derive(Debug)]
pub enum DataType {
    Null,
    Image(iced::widget::image::Handle),
    QrCode(iced::widget::qr_code::Data),
    Markdown(Vec<iced::widget::markdown::Item>),
    Text(String),
}

impl<'a> TryInto<&'a iced::widget::qr_code::Data> for &'a DataType {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a iced::widget::qr_code::Data, Self::Error> {
        if let DataType::QrCode(data) = self {
            Ok(data)
        } else {
            Err(ConversionError::InvalidType(
                "Expecting DataType::QrCode".into(),
            ))
        }
    }
}

impl<'a> TryInto<&'a Vec<Item>> for &'a DataType {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a Vec<iced::widget::markdown::Item>, Self::Error> {
        if let DataType::Markdown(data) = self {
            Ok(data)
        } else {
            Err(ConversionError::InvalidType(
                "Expecting DataType::Markdown".into(),
            ))
        }
    }
}

#[derive(Debug)]
pub enum DataProvider {
    None,
    File(FileProvider),
    QrCode(QRDataProvider),
}

impl<'a> TryInto<&'a iced::widget::qr_code::Data> for &'a DataProvider {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a iced::widget::qr_code::Data, Self::Error> {
        match self {
            DataProvider::QrCode(qr_data) => (&qr_data.0).try_into(),
            _ => Err(ConversionError::InvalidType(
                "Expecting DataProviders::QrCode".into(),
            )),
        }
    }
}

/*
impl<'a> TryInto<&'a Item> for &'a DataProvider {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a Item, Self::Error> {
        match self {
            DataProvider::File(file) => file.data().try_into(),
            _ => Err(ConversionError::InvalidType(
                "Expecting DataProvider::File".into(),
            )),
        }
    }
}
*/

impl<'a> IntoIterator for &'a DataProvider {
    type Item = &'a iced::widget::markdown::Item;

    type IntoIter = core::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DataProvider::File(file) => {
                if let DataType::Markdown(data) = file.data() {
                    data.iter()
                } else {
                    panic!("Expecting DataType::Markdown")
                }
            }
            _ => panic!("Expecting DataProvider::File"),
        }
    }
}

#[derive(Debug)]
pub struct FileProvider {
    path: PathBuf,
    data: Box<DataType>,
}

impl FileProvider {
    pub fn new(filename: &String) -> Self {
        info!("FileProvider filename='{filename}'");
        let path: PathBuf = filename.into();
        Self {
            path,
            data: Box::new(DataType::Null),
        }
    }

    pub fn load_text(&mut self) -> Result<(), Error> {
        info!("Loading file data from {:?}", self.path);
        let mut file = File::open(&self.path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        self.data = Box::new(DataType::Text(buf));

        Ok(())
    }

    pub fn load_markdown(&mut self) -> Result<(), Error> {
        info!("Loading markdown from {:?}", self.path);
        let mut file = File::open(&self.path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let items = DataType::Markdown(iced::widget::markdown::parse(&buf).collect());
        self.data = Box::new(items);

        Ok(())
    }

    pub fn data<'a>(&'a self) -> &'a DataType {
        &self.data
    }
}

#[derive(Debug)]
pub struct QRDataProvider(DataType);

impl QRDataProvider {
    pub fn new(data: iced::widget::qr_code::Data) -> Self {
        QRDataProvider(DataType::QrCode(data))
    }

    pub fn data<'a>(&'a self) -> &'a iced::widget::qr_code::Data {
        if let DataType::QrCode(data) = &self.0 {
            return data;
        } else {
            panic!("Expecting DataType::QrCode")
        }
    }
}
