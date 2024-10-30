use std::sync::Arc;

use crate::error::ConversionError;
use iced::widget::markdown::Item;

pub(crate) mod file_data;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod file_provider;
pub(crate) mod provider;
pub(crate) mod url_provider;

pub(crate) use file_data::FileData;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use file_provider::FileProvider;

#[derive(Debug)]
pub struct MarkdownItems(Arc<Vec<Item>>);

impl MarkdownItems {
    pub fn new(items: Vec<iced::widget::markdown::Item>) -> Self {
        MarkdownItems(Arc::new(items))
    }

    pub fn inner(&self) -> &Arc<Vec<Item>> {
        &self.0
    }

    pub fn into_iter<'a>(&'a self) -> impl IntoIterator<Item = &'a Item> + 'a {
        self.0.iter()
    }
}

impl Clone for MarkdownItems {
    fn clone(&self) -> Self {
        MarkdownItems(self.0.clone())
    }
}

#[derive(Debug)]
pub enum DataType {
    Null,
    Image(iced::widget::image::Handle),
    Svg(iced::widget::svg::Handle),
    QrCode(Arc<iced::widget::qr_code::Data>),
    Markdown(MarkdownItems),
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
            Ok(&data.0)
        } else {
            Err(ConversionError::InvalidType(
                "Expecting DataType::Markdown".into(),
            ))
        }
    }
}

impl<'a> TryInto<&'a iced::widget::svg::Handle> for &'a DataType {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a iced::widget::svg::Handle, Self::Error> {
        if let DataType::Svg(handle) = self {
            Ok(handle)
        } else {
            Err(ConversionError::InvalidType(
                "Expecting DataType::Svg".into(),
            ))
        }
    }
}
