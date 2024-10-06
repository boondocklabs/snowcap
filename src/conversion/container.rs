use iced::{widget::Container, Background, Element};

use crate::{
    attribute::Attributes,
    error::ConversionError,
    parser::{color::ColorParser, gradient::GradientParser, TreeNode},
    Message, Value,
};

pub struct SnowcapContainer;

impl SnowcapContainer {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        attrs: &Attributes,
        content: &'a TreeNode<AppMessage>,
    ) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: 'a + Clone + std::fmt::Debug,
    {
        let content: Element<'a, SnowcapMessage> = content.try_into()?;

        let mut container = Container::new(content);

        let mut style = iced::widget::container::Style::default();

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

                "bg" => {
                    if let Value::String(str) = value {
                        if let Ok(color) = ColorParser::parse_str(str) {
                            style.background = Some(Background::Color(color));
                        } else if let Ok(gradient) = GradientParser::parse_str(str) {
                            style.background = Some(Background::Gradient(gradient));
                        }
                    }
                    container
                }

                "text-color" => {
                    if let Value::String(str) = value {
                        let color = ColorParser::parse_str(str)?;
                        style.text_color = Some(color);
                    }
                    container
                }

                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            };
        }

        container = container.style(move |_theme| style);

        Ok(container.into())
    }
}
