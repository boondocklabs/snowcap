use std::sync::Arc;

use crate::attribute::Attributes;
use crate::data::DataType;
use crate::error::ConversionError;
use crate::message::WidgetMessage;
use crate::util::ElementWrapper;
use arbutus::NodeId;
use iced::{
    advanced::Widget,
    widget::{Image, QRCode, Svg, Text},
};

impl DataType {
    pub fn to_widget<'a, M>(
        node_id: NodeId,
        data_arc: Arc<DataType>,
        attrs: Attributes,
        //) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    ) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer>>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        match &*data_arc {
            DataType::Null => panic!("Null DataType"),
            DataType::Text(string) => {
                let mut text = Text::new(string.clone());

                for attr in attrs {}

                return Ok(Box::new(text));
            }
            DataType::Image(handle) => return Ok(Box::new(Image::new(handle))),
            DataType::Svg(handle) => return Ok(Box::new(Svg::new(handle.clone()))),
            DataType::QrCode(data) => {
                let mut qr = QRCode::new(data.clone());

                for attr in attrs {
                    qr = match *attr {
                        crate::attribute::AttributeValue::TextColor(color) => todo!(),
                        crate::attribute::AttributeValue::Border(border) => todo!(),
                        crate::attribute::AttributeValue::Shadow(shadow) => todo!(),
                        crate::attribute::AttributeValue::HorizontalAlignment(horizontal) => {
                            todo!()
                        }
                        crate::attribute::AttributeValue::VerticalAlignment(vertical) => todo!(),
                        crate::attribute::AttributeValue::Padding(padding) => todo!(),
                        crate::attribute::AttributeValue::WidthLength(length) => todo!(),
                        crate::attribute::AttributeValue::WidthPixels(pixels) => todo!(),
                        crate::attribute::AttributeValue::MaxWidth(pixels) => todo!(),
                        crate::attribute::AttributeValue::HeightLength(length) => todo!(),
                        crate::attribute::AttributeValue::HeightPixels(pixels) => todo!(),
                        crate::attribute::AttributeValue::Background(background) => todo!(),
                        crate::attribute::AttributeValue::Spacing(pixels) => todo!(),
                        crate::attribute::AttributeValue::Size(pixels) => todo!(),
                        crate::attribute::AttributeValue::CellSize(pixels) => qr.cell_size(pixels),
                        crate::attribute::AttributeValue::Clip(_) => todo!(),
                        crate::attribute::AttributeValue::Toggled(_) => todo!(),
                        crate::attribute::AttributeValue::Selected(_) => todo!(),
                        crate::attribute::AttributeValue::Label(_) => todo!(),
                    };
                }

                return Ok(Box::new(qr));
            }
            DataType::Markdown(markdown_items) => {
                let markdown: iced::Element<'static, M> = iced::widget::markdown(
                    markdown_items.into_iter(),
                    iced::widget::markdown::Settings::default(),
                    iced::widget::markdown::Style::from_palette(iced::Theme::default().palette()),
                )
                .map(move |url| M::from((node_id, WidgetMessage::Markdown(url))));

                Ok(Box::new(ElementWrapper::new(markdown)))
            }
        }
    }
}
