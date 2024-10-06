use iced::{widget::Stack, Element};

use crate::{attribute::Attributes, error::ConversionError, parser::TreeNode, Message};

pub struct SnowcapStack;

impl SnowcapStack {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        contents: &'a Vec<TreeNode<AppMessage>>,
    ) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: 'a + Clone + std::fmt::Debug,
    {
        let children: Result<Vec<Element<'a, SnowcapMessage>>, ConversionError> =
            contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

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
