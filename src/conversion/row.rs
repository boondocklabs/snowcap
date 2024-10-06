use iced::{widget::Row, Element};

use crate::{attribute::Attributes, error::ConversionError, parser::TreeNode, Message};

pub struct SnowcapRow;

impl SnowcapRow {
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

        let mut row = Row::with_children(children?);

        for attr in attrs {
            row = match attr.name().as_str() {
                "spacing" => {
                    let spacing: Result<iced::Pixels, ConversionError> =
                        (&*attr.value()).try_into();
                    row.spacing(spacing?)
                }
                "padding" => {
                    let padding: Result<iced::Padding, ConversionError> =
                        (&*attr.value()).try_into();
                    row.padding(padding?)
                }
                "width" => {
                    let width: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    row.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    row.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Vertical, ConversionError> =
                        (&*attr.value()).try_into();
                    row.align_y(align?)
                }
                "clip" => todo!(),
                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            };
        }

        Ok(row.into())
    }
}
