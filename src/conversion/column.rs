use iced::{widget::Column, Element};

use crate::{error::ConversionError, parser::Attributes, Error, MarkupTree, Message};

pub struct SnowcapColumn;

impl SnowcapColumn {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        contents: &'a Vec<MarkupTree<AppMessage>>,
    ) -> Result<Element<'a, SnowcapMessage>, Error>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
    {
        let children: Result<Vec<Element<'a, SnowcapMessage>>, Error> =
            contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

        let mut col = Column::with_children(children?);

        for attr in attrs {
            col = match attr.name.as_str() {
                "spacing" => todo!(),
                "padding" => todo!(),
                "width" => {
                    let width: Result<iced::Length, Error> = (&attr.value).try_into();
                    col.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, Error> = (&attr.value).try_into();
                    col.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Horizontal, Error> =
                        (&attr.value).try_into();
                    col.align_x(align?)
                }
                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        attr.name.clone(),
                    )))
                }
            }
        }

        Ok(col.into())
    }
}
