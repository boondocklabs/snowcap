use std::sync::Arc;

use iced::{widget::Stack, Element};
use tracing::info_span;

use crate::{
    attribute::Attributes, error::ConversionError, message::WidgetMessage, tree::node::TreeNode,
};

pub struct SnowcapStack<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    contents: Arc<Vec<TreeNode<'a, M>>>,
    stack: Stack<'a, M>,
}

impl<'a, M> SnowcapStack<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn convert(
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    ) -> Result<Stack<'a, M>, ConversionError>
    where
        //SnowcapMessage: Clone + From<Message<AppMessage>> + 'static,
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        let span = info_span!("stack");
        let _span = span.enter();

        let children: Result<Vec<Element<'a, M>>, ConversionError> = (**contents)
            .iter()
            .map(|item| item.clone().into_element())
            .collect(); // Convert each item into Element

        let mut stack = Stack::with_children(children?);

        for attr in attrs {
            stack = match attr.name().as_str() {
                "width" => {
                    let width: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    stack.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    stack.height(height?)
                }
                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            }
        }

        Ok(stack.into())
    }
}
