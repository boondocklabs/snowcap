use std::sync::Arc;

use iced::{widget::Container, Background};

use crate::{
    attribute::Attributes,
    error::ConversionError,
    message::WidgetMessage,
    parser::{color::ColorParser, gradient::GradientParser},
    tree::node::TreeNode,
    Value,
};

use super::widget::SnowcapWidget;

pub struct SnowcapContainer<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    container: Container<'a, M>,
    content: TreeNode<'a, M>,
}

impl<'a, M> SnowcapContainer<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn new(
        attrs: Attributes,
        content: TreeNode<'a, M>,
    ) -> Result<Container<'a, M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        let widget = SnowcapWidget::from_node(content.inner.borrow().clone())?;
        let mut content = content.clone();

        widget.map(|w| content.set_widget(w));

        let mut container = Container::new(content.into_element()?);

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

        Ok(container)
    }
}
