use iced::{widget::Column, Element};

use crate::{attribute::Attributes, error::ConversionError, parser::TreeNode, Message};

pub struct SnowcapColumn;

impl SnowcapColumn {
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

        let mut col = Column::with_children(children?);

        for attr in attrs {
            col = match attr.name().as_str() {
                "spacing" => todo!(),
                "padding" => todo!(),
                "width" => {
                    let width: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    col.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    col.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Horizontal, ConversionError> =
                        (&*attr.value()).try_into();
                    col.align_x(align?)
                }
                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            }
        }

        Ok(col.into())
    }
}
