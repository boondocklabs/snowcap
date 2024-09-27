use iced::{widget::Row, Element};

use crate::{error::ConversionError, parser::Attributes, Error, MarkupTree, Message};

pub struct SnowcapRow;

impl SnowcapRow {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        contents: &'a Vec<MarkupTree<AppMessage>>,
    ) -> Result<Element<'a, SnowcapMessage>, Error>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: std::fmt::Debug,
    {
        let children: Result<Vec<Element<'a, SnowcapMessage>>, Error> =
            contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

        let mut row = Row::with_children(children?);

        for attr in attrs {
            row = match attr.name().as_str() {
                "spacing" => {
                    let spacing: Result<iced::Pixels, Error> = (&*attr.value()).try_into();
                    row.spacing(spacing?)
                }
                "padding" => {
                    let padding: Result<iced::Padding, Error> = (&*attr.value()).try_into();
                    row.padding(padding?)
                }
                "width" => {
                    let width: Result<iced::Length, Error> = (&*attr.value()).try_into();
                    row.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, Error> = (&*attr.value()).try_into();
                    row.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Vertical, Error> =
                        (&*attr.value()).try_into();
                    row.align_y(align?)
                }
                "clip" => todo!(),
                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        attr.name().clone(),
                    )))
                }
            };
        }

        Ok(row.into())
    }
}
