use std::sync::Arc;

use crate::attribute::Attributes;
use crate::data::DataType;
use crate::error::ConversionError;
use crate::message::Message;
use iced::widget::markdown::{Settings, Style};
use iced::widget::{Image, QRCode, Svg, Text};
use iced::{Element, Theme};

impl DataType {
    pub fn to_widget<'a, SnowcapMessage, AppMessage>(
        data_arc: Arc<DataType>,
        attrs: &'a Attributes,
    ) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: 'a + Clone + std::fmt::Debug,
    {
        match &*data_arc {
            DataType::Null => panic!("Null DataType"),
            DataType::Text(string) => {
                let mut text = Text::new(string.clone());

                for attr in attrs {
                    match attr.name().as_str() {
                        "size" => {
                            let pixels: iced::Pixels = attr.try_into()?;

                            // cosmic-text will panic if the text size is too small
                            if pixels.0 < 2.0 {
                                tracing::warn!("text size too small. clamping.");
                                text = text.size(2.0);
                            } else {
                                text = text.size(pixels);
                            }
                        }
                        _ => {}
                    };
                }

                Ok(text.into())
            }
            DataType::Image(handle) => Ok(Image::new(handle).into()),
            DataType::Svg(handle) => Ok(Svg::new(handle.clone()).into()),
            DataType::QrCode(data) => {
                let mut qr = QRCode::new(data.clone());
                for attr in attrs {
                    qr = match attr.name().as_str() {
                        "cell-size" => {
                            let cell_size: u16 = attr.try_into()?;
                            qr.cell_size(cell_size)
                        }
                        _ => qr,
                    };
                }

                Ok(qr.into())
            }
            DataType::Markdown(markdown_items) => Ok(iced::widget::markdown(
                markdown_items.into_iter(),
                Settings::default(),
                Style::from_palette(Theme::default().palette()),
            )
            .map(|url| SnowcapMessage::from(Message::Markdown(url)))
            .into()),
        }
    }
}

/*
impl<'a, SnowcapMessage> TryInto<Element<'a, SnowcapMessage>> for &'a DataType
where
    SnowcapMessage: 'a + Clone,
{
    type Error = ConversionError;

    fn try_into(self) -> Result<Element<'a, SnowcapMessage>, Self::Error> {
        match self {
            DataType::Null => panic!("Null DataType"),
            DataType::Image(handle) => Ok(Image::new(handle).into()),
            DataType::Svg(handle) => Ok(Svg::new(handle.clone()).into()),
            DataType::QrCode(data) => Ok(QRCode::new(data).into()),
            DataType::Markdown(markdown_items) => Ok(iced::widget::markdown(
                markdown_items.into_iter(),
                Settings::default(),
                Style::from_palette(Theme::default().palette()),
            )
            .map(|url| Message::Markdown(url))
            .into()),
            DataType::Text(_) => todo!(),
        }
    }
}
*/
