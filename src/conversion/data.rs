use crate::data::DataType;
use crate::error::ConversionError;
use crate::message::WidgetMessage;
use crate::util::ElementWrapper;
use crate::{attribute::Attributes, dynamic_widget::DynamicWidget};
use arbutus::NodeId;
use iced::widget::{Image, QRCode, Svg, Text};
use tracing::warn;

impl DataType {
    pub fn to_widget<'a, M>(
        self,
        node_id: NodeId,
        attrs: Attributes,
        //) -> Result<Element<'a, SnowcapMessage>, ConversionError>
        //) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer>>, ConversionError>
    ) -> Result<DynamicWidget<'a, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        match self {
            DataType::Null => panic!("Null DataType"),
            DataType::Text(string) => {
                let text = Text::new(string.clone());

                //for attr in attrs {}

                return Ok(DynamicWidget::default().with_widget(text));
            }
            DataType::Image(handle) => {
                return Ok(DynamicWidget::default().with_widget(Image::new(handle)))
            }
            DataType::Svg(handle) => {
                return Ok(DynamicWidget::default().with_widget(Svg::new(handle.clone())))
            }
            DataType::QrCode(data) => {
                let mut qr = QRCode::new(data.clone());

                for attr in attrs {
                    qr = match *attr {
                        crate::attribute::AttributeValue::CellSize(pixels) => qr.cell_size(pixels),
                        _ => {
                            warn!("Unsupported QRCode attribute {:?}", attr);
                            qr
                        }
                    };
                }

                return Ok(DynamicWidget::default().with_widget(qr));
            }
            DataType::Markdown(markdown_items) => {
                let markdown: iced::Element<'static, M> = iced::widget::markdown(
                    markdown_items.into_iter(),
                    iced::widget::markdown::Settings::default(),
                    iced::widget::markdown::Style::from_palette(iced::Theme::default().palette()),
                )
                .map(move |url| M::from((node_id, WidgetMessage::Markdown(url))));

                Ok(DynamicWidget::default().with_widget(ElementWrapper::new(markdown)))
            }
        }
    }
}
