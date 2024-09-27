use iced::{widget::Container, Element};

use crate::{error::ConversionError, parser::Attributes, Error, MarkupTree, Message};

pub struct SnowcapContainer;

impl SnowcapContainer {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        content: &'a MarkupTree<AppMessage>,
    ) -> Result<Element<'a, SnowcapMessage>, Error>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: std::fmt::Debug,
    {
        let content: Element<'a, SnowcapMessage> = content.try_into()?;

        let mut container = Container::new(content);

        for attr in attrs {
            let value = &*attr.value();
            container = match attr.name().as_str() {
                "padding" => {
                    let padding: iced::Padding = value.try_into()?;
                    container.padding(padding)
                }

                "width" => {
                    let width: iced::Length = value.try_into()?;
                    container.width(width)
                }

                "height" => {
                    let height: iced::Length = value.try_into()?;
                    container.height(height)
                }

                "max-width" => {
                    let width: iced::Pixels = value.try_into()?;
                    container.max_width(width)
                }

                "max-height" => {
                    let height: iced::Pixels = value.try_into()?;
                    container.max_height(height)
                }

                "align-x" => {
                    let align: iced::alignment::Horizontal = value.try_into()?;
                    container.align_x(align)
                }

                "align-y" => {
                    let align: iced::alignment::Vertical = value.try_into()?;
                    container.align_y(align)
                }

                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        attr.name().clone(),
                    )))
                }
            };
        }

        Ok(container.into())
    }
}
