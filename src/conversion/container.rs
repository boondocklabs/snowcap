use crate::{attribute::AttributeValue, NodeId};
use iced::widget::Container;

use crate::{
    attribute::Attributes, dynamic_widget::DynamicWidget, error::ConversionError,
    message::WidgetMessage,
};

pub struct SnowcapContainer;

impl SnowcapContainer {
    pub fn new<M>(
        attrs: Attributes,
        content: DynamicWidget<'static, M>,
        //) -> Result<Container<'a, M>, ConversionError>
    ) -> Result<DynamicWidget<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)>,
    {
        let mut container = Container::new(content.into_element());
        let mut style = iced::widget::container::Style::default();

        for attr in attrs {
            (container, style) = match *attr {
                AttributeValue::TextColor(color) => (container, style.color(color)),
                AttributeValue::Border(border) => (container, style.border(border)),
                AttributeValue::Shadow(shadow) => (container, style.shadow(shadow)),
                AttributeValue::Background(background) => (container, style.background(background)),
                AttributeValue::HorizontalAlignment(horizontal) => {
                    (container.align_x(horizontal), style)
                }
                AttributeValue::VerticalAlignment(vertical) => (container.align_y(vertical), style),
                AttributeValue::Padding(padding) => (container.padding(padding), style),
                AttributeValue::MaxWidth(pixels) => (container.max_width(pixels), style),
                AttributeValue::WidthLength(length) => (container.width(length), style),
                AttributeValue::HeightLength(length) => (container.height(length), style),
                AttributeValue::WidthPixels(pixels) => (container.width(pixels), style),
                AttributeValue::HeightPixels(pixels) => (container.height(pixels), style),
                AttributeValue::Clip(clip) => (container.clip(clip), style),
                _ => {
                    return Err(ConversionError::UnsupportedAttribute(
                        attr,
                        "Container".into(),
                    ))
                }
            };
        }

        container = container.style(move |_theme| style);

        Ok(DynamicWidget::default().with_widget(container))
    }
}
