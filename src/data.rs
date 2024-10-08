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

/*
#[derive(Debug)]
pub enum DataProvider {
    None,
    File(FileProvider),
    QrCode(QRDataProvider),
    Url(UrlProvider),
}

impl Provider for DataProvider {
    fn update_task(&self) -> iced::Task<Event> {
        match self {
            DataProvider::None => todo!(),
            DataProvider::File(file_provider) => file_provider.update_task(),
            DataProvider::QrCode(qrdata_provider) => qrdata_provider.update_task(),
            DataProvider::Url(url_provider) => url_provider.update_task(),
        }
    }

    fn init_task(&self) -> Task<Event> {
        match self {
            DataProvider::None => todo!(),
            DataProvider::File(file_provider) => file_provider.update_task(),
            DataProvider::QrCode(qrdata_provider) => qrdata_provider.update_task(),
            DataProvider::Url(url_provider) => url_provider.update_task(),
        }
    }

    fn set_event_inlet(&self, inlet: Inlet<Event>) {
        match self {
            DataProvider::File(p) => p.set_event_inlet(inlet),
            DataProvider::QrCode(p) => p.set_event_inlet(inlet),
            DataProvider::Url(p) => p.set_event_inlet(inlet),
            _ => {} //_ => Err(Error::Unhandled("DataProvider update request".into())),
        }
    }
}
*/

/*
impl<'a> TryInto<&'a iced::widget::qr_code::Data> for &'a DataProvider {
    type Error = ConversionError;

    fn try_into(self) -> Result<&'a iced::widget::qr_code::Data, Self::Error> {
        match self {
            DataProvider::QrCode(qr_data) => (&qr_data.data).try_into(),
            _ => Err(ConversionError::InvalidType(
                "Expecting DataProviders::QrCode".into(),
            )),
        }
    }
}
*/

/*
impl<'a> IntoIterator for &'a DataProvider {
    type Item = &'a iced::widget::markdown::Item;

    type IntoIter = core::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DataProvider::File(file) => {
                if let DataType::Markdown(data) = file.lock().expect("Failed to lock mutex").data()
                {
                    data.iter()
                } else {
                    panic!("Expecting DataType::Markdown")
                }
            }
            _ => panic!("Expecting DataProvider::File"),
        }
    }
}
*/

/*
#[derive(Debug)]
pub struct QRDataProvider {
    data: DataType,
}

impl QRDataProvider {
    pub fn new(data: iced::widget::qr_code::Data) -> Self {
        QRDataProvider {
            data: DataType::QrCode(data),
        }
    }

    pub fn data<'a>(&'a self) -> &'a iced::widget::qr_code::Data {
        if let DataType::QrCode(data) = &self.data {
            return data;
        } else {
            panic!("Expecting DataType::QrCode")
        }
    }
}

impl Provider for QRDataProvider {
    fn set_event_inlet(&self, _inlet: Inlet<Event>) {}

    fn update_task(&self, node_id: NodeId) -> Task<Event> {
        Task::none()
    }

    fn init_task(&self) -> Task<Event> {
        Task::none()
    }
}
*/
