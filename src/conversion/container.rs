use crate::{attribute::AttributeValue, NodeId, NodeRef};
use iced::{widget::Container, Background};

use crate::{
    attribute::Attributes,
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    parser::{color::ColorParser, gradient::GradientParser},
    Value,
};

pub struct SnowcapContainer<'a, M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    container: Container<'a, M>,
    content: &'a NodeRef<M>,
}

impl<'a, M> SnowcapContainer<'a, M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub fn new(
        attrs: Attributes,
        content: NodeRef<M>,
    ) -> Result<Container<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)>,
    {
        //let widget = SnowcapWidget::from_node(content.inner.borrow().clone())?;
        //let mut content = content.clone();
        //widget.map(|w| content.set_widget(w));

        let content = DynamicWidget::from_node(content)?.into_element();
        let mut container = Container::new(content);
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

        Ok(container)
    }
}
