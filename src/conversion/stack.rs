use iced::{widget::Stack, Element};

use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    tree_util::ChildData,
    NodeId,
};

pub struct SnowcapStack;

impl SnowcapStack {
    pub fn convert<M>(
        attrs: Attributes,
        contents: Option<Vec<ChildData<'static, M>>>,
    ) -> Result<DynamicWidget<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut stack = if let Some(contents) = contents {
            let children: Vec<Element<'_, M>> = contents
                .into_iter()
                .filter_map(|item| {
                    tracing::debug!("Stack Child {:?}", item);
                    Some(item.into())
                })
                .collect(); // Convert each item into Element

            Stack::with_children(children)
        } else {
            Stack::new()
        };

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
