use iced::{
    widget::{
        text::IntoFragment, Button, Column, Container, QRCode, Row, Rule, Space, Stack, Text,
        Toggler,
    },
    Element, Length, Pixels,
};
use tracing::debug;

use crate::{
    data::{DataProvider, DataType},
    error::{ConversionError, Error},
    message::Message,
    parser::Value,
    MarkupType,
};

impl<'a> IntoFragment<'a> for &MarkupType {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        match self {
            MarkupType::Value(value) => match value {
                Value::String(s) => s.clone().into(),
                Value::Number(n) => format!("{n}").into(),
                Value::Boolean(b) => format!("{b}").into(),
                Value::Null => format!("null").into(),
                Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                } => match provider {
                    //DataProvider::File(file_provider) => file_provider.data().clone().into(),
                    _ => "Unsupported DataProvider".into(),
                },
            },
            _ => "Expecting MarkupType::Value".into(),
        }
    }
}

impl<'a, M> TryInto<Element<'a, M>> for &'a MarkupType
where
    M: 'a + Clone + From<Message>,
{
    type Error = Error;

    fn try_into(self) -> Result<Element<'a, M>, Error> {
        match self {
            MarkupType::None => Ok(Space::new(0, 0).into()),
            MarkupType::Element {
                name,
                attrs,
                content,
            } => match name.as_str() {
                "text" => {
                    let mut text = Text::new(&**content);

                    for attr in attrs {
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
                    let width: Length = if let Some(width) = attrs.get("width") {
                        width.try_into()?
                    } else {
                        Length::Fixed(0.0)
                    };

                    let height: Length = if let Some(height) = attrs.get("height") {
                        height.try_into()?
                    } else {
                        Length::Fixed(0.0)
                    };

                    let space = Space::new(width, height);

                    Ok(space.into())
                }

                "button" => {
                    let content: Element<'a, M> = (&**content).try_into()?;

                    let button = Button::new(content);

                    Ok(button.into())
                }

                "rule-horizontal" => {
                    let height: Pixels = attrs
                        .get("height")
                        .ok_or_else(|| Error::MissingAttribute("height".to_string()))?
                        .try_into()?;
                    debug!("[Rule horizontal height={height:?}]");
                    Ok(Rule::horizontal(height).into())
                }

                "rule-vertical" => {
                    let width: Pixels = attrs
                        .get("width")
                        .ok_or_else(|| Error::MissingAttribute("height".to_string()))?
                        .try_into()?;
                    Ok(Rule::vertical(width).into())
                }

                "toggler" => {
                    let is_toggled: bool = if let Some(width) = attrs.get("toggled") {
                        width.try_into()?
                    } else {
                        false
                    };

                    let mut toggler = Toggler::new(is_toggled);

                    for attr in attrs {
                        toggler = match attr.name.as_str() {
                            "size" => {
                                let pixels: iced::Pixels = attr.try_into()?;
                                toggler.size(pixels)
                            }
                            _ => toggler,
                        };
                    }

                    Ok(toggler.into())
                }

                "qr-code" => {
                    if let MarkupType::Value(Value::DataSource {
                        name: _,
                        value: _,
                        provider,
                    }) = &**content
                    {
                        let mut qr: QRCode = QRCode::new(provider.try_into()?);

                        for attr in attrs {
                            qr = match attr.name.as_str() {
                                "cell-size" => {
                                    let cell_size: u16 = attr.try_into()?;
                                    qr.cell_size(cell_size)
                                }
                                _ => qr,
                            };
                        }

                        Ok(qr.into())
                    } else {
                        Err(Error::Conversion(ConversionError::Missing("data".into())))
                    }
                }
                "markdown" => {
                    if let MarkupType::Value(Value::DataSource {
                        name: _,
                        value: _,
                        provider,
                    }) = &**content
                    {
                        if let DataProvider::File(file) = provider {
                            //file.load_markdown().unwrap();
                            if let DataType::Markdown(data) = file.data() {
                                let element = iced::widget::markdown::view(
                                    data,
                                    iced::widget::markdown::Settings::default(),
                                    iced::widget::markdown::Style::from_palette(
                                        iced::Theme::TokyoNightStorm.palette(),
                                    ),
                                );

                                Ok(element.map(|_url| M::from(Message::Markdown)).into())
                            } else {
                                panic!("Expecting DataType::Markdown")
                            }
                        } else {
                            panic!("Expect DataProvider::File");
                        }
                    } else {
                        panic!("Expect MarkupType::Value")
                    }
                }
                _ => {
                    return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                        format!("Unhandled element type {name}"),
                    )))
                }
            },
            MarkupType::Container { content, attrs } => {
                let content: Element<'a, M> = (&**content).try_into()?;

                let mut container = Container::new(content);

                for attr in attrs {
                    let value = &attr.value;
                    container = match attr.name.as_str() {
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
                                attr.name.clone(),
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
                            let width: Result<iced::Length, Error> = (&attr.value).try_into();
                            row.width(width?)
                        }
                        "height" => {
                            let height: Result<iced::Length, Error> = (&attr.value).try_into();
                            row.height(height?)
                        }
                        "align" => {
                            let align: Result<iced::alignment::Vertical, Error> =
                                (&attr.value).try_into();
                            row.align_y(align?)
                        }
                        "clip" => todo!(),
                        _ => {
                            return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                                attr.name.clone(),
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
            MarkupType::Stack { attrs, contents } => {
                let children: Result<Vec<Element<'a, M>>, Error> =
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
            MarkupType::Label(_) => todo!(),
            MarkupType::Value(value) => {
                // Convert Values to iced Elements
                match value {
                    Value::String(str) => Ok(Text::new(str.clone()).into()),
                    Value::Number(num) => Ok(Text::new(num).into()),
                    Value::Boolean(val) => Ok(Text::new(val).into()),
                    Value::Null => Ok(Text::new("null").into()),
                    Value::DataSource {
                        name,
                        value,
                        provider: _,
                    } => Ok(Text::new(format!("Data source [{name}:{value}]")).into()),
                }
            }
        }
    }
}
