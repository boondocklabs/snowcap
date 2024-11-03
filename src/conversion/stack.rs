use iced::widget::Stack;

use crate::{
    attribute::{AttributeValue, Attributes},
    cache::WidgetContent,
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    NodeId,
};

pub struct SnowcapStack;

impl SnowcapStack {
    pub fn convert<M>(
        attrs: Attributes,
        contents: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut stack = Stack::with_children(contents);

        for attr in attrs {
            stack = match *attr.value() {
                AttributeValue::WidthLength(length) => stack.width(length),
                AttributeValue::HeightLength(length) => stack.height(length),
                _ => return Err(ConversionError::UnsupportedAttribute(attr, "Stack".into())),
            };
        }

        Ok(DynamicWidget::default().with_widget(stack))
    }
}
