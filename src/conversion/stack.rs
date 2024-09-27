use iced::{widget::Stack, Element};

use crate::{error::ConversionError, parser::Attributes, Error, MarkupTree, Message};

pub struct SnowcapStack;

impl SnowcapStack {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        contents: &'a Vec<MarkupTree<AppMessage>>,
    ) -> Result<Element<'a, SnowcapMessage>, Error>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
    {
        let children: Result<Vec<Element<'a, SnowcapMessage>>, Error> =
            contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

        let mut stack = Stack::with_children(children?);

        for attr in attrs {
            stack = match attr.name.as_str() {
                "width" => {
                    let width: Result<iced::Length, Error> = (&attr.value).try_into();
                    stack.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, Error> = (&attr.value).try_into();
                    stack.height(height?)
                }
                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        attr.name.clone(),
                    )))
                }
            }
        }

        Ok(stack.into())
    }
}
