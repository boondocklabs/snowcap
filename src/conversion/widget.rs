use std::sync::Mutex;

use iced::widget::{Button, Image, PickList, QRCode, Rule, Space, Svg, Themer, Toggler};
use iced::{widget::Text, Element};
use iced::{Length, Pixels, Theme};
use once_cell::sync::Lazy;
use tracing::{debug, info};

use crate::data::{DataProvider, DataType};
use crate::error::ConversionError;
use crate::message::Message;
use crate::parser::Attributes;
use crate::parser::Value;
use crate::{Error, MarkupTree};

pub struct SnowcapWidget;

impl SnowcapWidget {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        name: &String,
        attrs: &'a Attributes,
        content: &'a MarkupTree<AppMessage>,
    ) -> Result<Element<'a, SnowcapMessage>, Error>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: 'a + Clone + std::fmt::Debug,
    {
        match name.as_str() {
            "text" => {
                let mut text = Text::new(content);

                for attr in attrs {
                    match attr.name().as_str() {
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
                let content: Element<'a, SnowcapMessage> = content.try_into()?;

                let button =
                    Button::new(content).on_press_with(|| SnowcapMessage::from(Message::Button));

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

                let mut toggler = Toggler::new(is_toggled).on_toggle(|toggled| {
                    attrs.set("toggled", Value::Boolean(toggled));
                    SnowcapMessage::from(Message::Toggler(toggled))
                });

                for attr in attrs {
                    toggler = match attr.name().as_str() {
                        "size" => {
                            let pixels: iced::Pixels = attr.try_into()?;
                            toggler.size(pixels)
                        }
                        "label" => toggler.label(attr),
                        _ => toggler,
                    };
                }

                Ok(toggler.into())
            }

            "qr-code" => {
                if let MarkupTree::Value(Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                }) = content
                {
                    let mut qr: QRCode = QRCode::new(provider.try_into()?);

                    for attr in attrs {
                        qr = match attr.name().as_str() {
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
                if let MarkupTree::Value(Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                }) = content
                {
                    if let DataProvider::File(file) = provider {
                        if let DataType::Markdown(data) = file.data() {
                            let element = iced::widget::markdown::view(
                                data,
                                iced::widget::markdown::Settings::default(),
                                iced::widget::markdown::Style::from_palette(
                                    iced::Theme::TokyoNightStorm.palette(),
                                ),
                            );

                            Ok(element
                                .map(|url| SnowcapMessage::from(Message::Markdown(url)))
                                .into())
                        } else {
                            panic!("Expecting DataType::Markdown")
                        }
                    } else {
                        panic!("Expect DataProvider::File");
                    }
                } else {
                    panic!("Expect MarkupType::Value::DataSource")
                }
            }
            "image" => {
                if let MarkupTree::Value(Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                }) = content
                {
                    if let DataProvider::File(file) = provider {
                        if let DataType::Image(handle) = file.data() {
                            let image = Image::new(handle);
                            Ok(image.into())
                        } else {
                            panic!("Expecting DataType::Image")
                        }
                    } else {
                        panic!("Expecting DataProvider::File")
                    }
                } else {
                    panic!("Expect MarkupType::Value::DataSource")
                }
            }
            "themer" => {
                // Create an iced::Theme from the name in the "theme" attribute
                let theme: Theme = attrs
                    .get("theme")
                    .ok_or_else(|| Error::MissingAttribute("theme".to_string()))?
                    .try_into()?;

                let content: Element<'a, SnowcapMessage, Theme> = content.try_into()?;

                let themer = Themer::new(
                    move |old_theme| {
                        info!("Themer from {:?} to {:?}", old_theme, theme);
                        theme.clone()
                    },
                    content,
                );
                Ok(themer.into())
            }
            "svg" => {
                if let MarkupTree::Value(Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                }) = content
                {
                    if let DataProvider::File(file) = provider {
                        if let DataType::Svg(handle) = file.data() {
                            let svg = Svg::new(handle.clone());
                            Ok(svg.into())
                        } else {
                            panic!("Expecting DataType::Svg")
                        }
                    } else {
                        panic!("Expecting DataProvider::File")
                    }
                } else {
                    panic!("Expect MarkupTree::Value::DataSource")
                }
            }
            "pick-list" => match content {
                MarkupTree::Value(value) => {
                    if let Value::Array(values) = value {
                        let values: Result<Vec<String>, ConversionError> =
                            values.iter().map(|v| v.try_into()).collect();

                        static CURRENT_SELECTION: Lazy<Mutex<Option<String>>> =
                            Lazy::new(move || Mutex::new(None));

                        let current = match &*CURRENT_SELECTION.lock().unwrap() {
                            Some(selection) => selection.clone(),
                            None => {
                                // Return the first item as the inital selection
                                values.as_ref().unwrap().first().unwrap().clone()
                            }
                        };

                        let picklist = PickList::new(values?, Some(current), |selected| {
                            // Store the selected item in the static
                            *CURRENT_SELECTION.lock().unwrap() = Some(selected.clone());
                            SnowcapMessage::from(Message::PickListSelected(selected.into()))
                        });

                        Ok(picklist.into())
                    } else {
                        panic!("Expecting Value::Array")
                    }
                }
                _ => panic!("Expecting MarkupTree::Value"),
            },
            _ => {
                return Err(Error::Conversion(ConversionError::UnsupportedAttribute(
                    format!("Unhandled element type {name}"),
                )))
            }
        }
    }
}
