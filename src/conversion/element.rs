use iced::{
    widget::{Button, Column, Container, Row, Rule, Space, Stack, Text},
    Element, Length, Pixels,
};
use tracing::debug;

use crate::{
    error::{ConversionError, Error},
    parser::{Attribute, Value},
    MarkupType,
};

impl<'a, M> TryInto<Element<'a, M>> for MarkupType
where
    M: 'a + Clone,
{
    type Error = Error;

    fn try_into(self) -> Result<Element<'a, M>, Error> {
        match self {
            MarkupType::None => Ok(Space::new(0, 0).into()),
            MarkupType::Element {
                name,
                attrs: attr,
                content,
            } => match name.as_str() {
                "text" => {
                    let mut text = if let MarkupType::Value(Value::String(str)) = *content {
                        Text::new(str)
                    } else {
                        Text::new("Bad value")
                    };

                    for attr in attr {
                        match attr.name.as_str() {
                            "size" => {
                                let pixels: iced::Pixels = attr.try_into()?;
                                text = text.size(pixels);
                            }
                            _ => {}
                        };
                    }

                    Ok(text.into())
                }

                "space" => {
                    let width: Length = attr
                        .get("width")
                        .unwrap_or(Attribute {
                            name: "width".to_string(),
                            value: Value::Number(0.0),
                        })
                        .try_into()?;

                    let height: Length = attr
                        .get("height")
                        .unwrap_or(Attribute {
                            name: "height".to_string(),
                            value: Value::Number(0.0),
                        })
                        .try_into()?;

                    let space = Space::new(width, height);

                    Ok(space.into())
                }

                "button" => {
                    let content: Element<'a, M> = (*content).try_into()?;

                    let button = Button::new(content);

                    Ok(button.into())
                }

                "rule-horizontal" => {
                    let height: Pixels = attr
                        .get("height")
                        .ok_or_else(|| Error::MissingAttribute("height".to_string()))?
                        .try_into()?;
                    debug!("[Rule horizontal height={height:?}]");
                    Ok(Rule::horizontal(height).into())
                }

                "rule-vertical" => {
                    let width: Pixels = attr
                        .get("width")
                        .ok_or_else(|| Error::MissingAttribute("height".to_string()))?
                        .try_into()?;
                    Ok(Rule::vertical(width).into())
                }

                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        format!("Unhandled element type {name}"),
                    )))
                }
            },
            MarkupType::Container { content, attrs } => {
                let content: Element<'a, M> = (*content).try_into()?;

                let mut container = Container::new(content);

                for attr in attrs {
                    container = match attr.name.as_str() {
                        "padding" => {
                            let padding: Result<iced::Padding, Error> = attr.value.try_into();
                            container.padding(padding?)
                        }

                        "width" => {
                            let width: Result<iced::Length, Error> = attr.value.try_into();
                            container.width(width?)
                        }

                        "height" => {
                            let height: Result<iced::Length, Error> = attr.value.try_into();
                            container.height(height?)
                        }

                        "max-width" => {
                            let width: Result<iced::Pixels, Error> = attr.value.try_into();
                            container.max_width(width?)
                        }

                        "max-height" => {
                            let height: Result<iced::Pixels, Error> = attr.value.try_into();
                            container.max_height(height?)
                        }

                        "align-x" => {
                            let align: Result<iced::alignment::Horizontal, Error> =
                                attr.value.try_into();
                            container.align_x(align?)
                        }

                        "align-y" => {
                            let align: Result<iced::alignment::Vertical, Error> =
                                attr.value.try_into();
                            container.align_y(align?)
                        }

                        _ => {
                            return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                                attr.name,
                            )))
                        }
                    };
                }

                Ok(container.into())
            }
            MarkupType::Row { attrs, contents } => {
                let children: Result<Vec<Element<'a, M>>, Error> =
                    contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

                let mut row = Row::with_children(children?);

                for attr in attrs {
                    row = match attr.name.as_str() {
                        "spacing" => todo!(),
                        "padding" => todo!(),
                        "width" => {
                            let width: Result<iced::Length, Error> = attr.value.try_into();
                            row.width(width?)
                        }
                        "height" => {
                            let height: Result<iced::Length, Error> = attr.value.try_into();
                            row.height(height?)
                        }
                        "align" => {
                            let align: Result<iced::alignment::Vertical, Error> =
                                attr.value.try_into();
                            row.align_y(align?)
                        }
                        "clip" => todo!(),
                        _ => {
                            return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                                attr.name,
                            )))
                        }
                    };
                }

                Ok(row.into())
            }
            MarkupType::Column { attrs, contents } => {
                let children: Result<Vec<Element<'a, M>>, Error> =
                    contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

                let mut col = Column::with_children(children?);

                for attr in attrs {
                    col = match attr.name.as_str() {
                        "spacing" => todo!(),
                        "padding" => todo!(),
                        "width" => {
                            let width: Result<iced::Length, Error> = attr.value.try_into();
                            col.width(width?)
                        }
                        "height" => {
                            let height: Result<iced::Length, Error> = attr.value.try_into();
                            col.height(height?)
                        }
                        "align" => {
                            let align: Result<iced::alignment::Horizontal, Error> =
                                attr.value.try_into();
                            col.align_x(align?)
                        }
                        _ => {
                            return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                                attr.name,
                            )))
                        }
                    }
                }

                Ok(col.into())
            }
            MarkupType::Stack { attrs, contents } => {
                let children: Result<Vec<Element<'a, M>>, Error> =
                    contents.into_iter().map(|item| item.try_into()).collect(); // Convert each item into Element

                let mut stack = Stack::with_children(children?);

                for attr in attrs {
                    stack = match attr.name.as_str() {
                        "width" => {
                            let width: Result<iced::Length, Error> = attr.value.try_into();
                            stack.width(width?)
                        }
                        "height" => {
                            let height: Result<iced::Length, Error> = attr.value.try_into();
                            stack.height(height?)
                        }
                        _ => {
                            return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                                attr.name,
                            )))
                        }
                    }
                }

                Ok(stack.into())
            }
            MarkupType::Label(_) => todo!(),
            MarkupType::Value(value) => {
                // Convert Values to iced Elements
                match value {
                    Value::String(str) => Ok(Text::new(str).into()),
                    Value::Number(num) => Ok(Text::new(num).into()),
                    Value::Boolean(val) => Ok(Text::new(val).into()),
                    Value::Null => Ok(Text::new("null").into()),
                }
            }
        }
    }
}
