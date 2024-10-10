use std::sync::Arc;

use crate::attribute::Attributes;
use crate::data::DataType;
use crate::error::ConversionError;
use iced::{
    advanced::Widget,
    widget::{Image, QRCode, Svg, Text},
};

impl DataType {
    pub fn to_widget<'a, M>(
        data_arc: Arc<DataType>,
        attrs: Attributes,
        //) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    ) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer>>, ConversionError>
    where
        M: Clone + std::fmt::Debug + 'a,
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

                return Ok(Box::new(text));
            }
            DataType::Image(handle) => return Ok(Box::new(Image::new(handle))),
            DataType::Svg(handle) => return Ok(Box::new(Svg::new(handle.clone()))),
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

                return Ok(Box::new(qr));
            }
            DataType::Markdown(_markdown_items) => {
                return Ok(Box::new(Text::new("todo")));
                /*
                return Ok(iced::widget::markdown(
                    markdown_items.into_iter(),
                    Settings::default(),
                    Style::from_palette(Theme::default().palette()),
                )
                .map(|url| SnowcapMessage::from(Message::Markdown(url))))
                */
            }
        }
    }
}
