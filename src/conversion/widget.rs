use iced::widget::{Button, PickList, Rule, Space, Themer, Toggler};
use iced::{widget::Text, Element};
use iced::{Length, Pixels, Theme};
use tracing::{debug, info, warn};

use crate::attribute::Attributes;
use crate::error::ConversionError;
use crate::message::Message;
use crate::parser::TreeNode;
use crate::parser::Value;
use crate::MarkupTree;

pub struct SnowcapWidget;

impl SnowcapWidget {
    pub fn convert<'a, SnowcapMessage, AppMessage>(
        node: &'a TreeNode<AppMessage>,
        name: &String,
        attrs: &'a Attributes,
        content: &'a TreeNode<AppMessage>,
    ) -> Result<Element<'a, SnowcapMessage>, ConversionError>
    where
        SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
        AppMessage: 'a + Clone + std::fmt::Debug,
    {
        // Handle any nodes with a value of Value::Data
        // as we can infer the widget type from the DataType
        // using .to_widget()
        match content.inner() {
            MarkupTree::Value(value) => match &*value.borrow() {
                Value::Data {
                    data: Some(data), ..
                } => {
                    let arc = (*data).clone();
                    return crate::data::DataType::to_widget(arc, attrs);
                }
                Value::Data { data: None, .. } => {
                    // No data is available, so provide a placeholder widget
                    // TODO: Some kind of spinner?
                    return Ok(Text::new("Data Not loaded").into());
                }
                _ => {}
            },
            /*
            MarkupTree::Value(Value::Data {
                data: Some(data), ..
            }) => {
                return data.to_widget(attrs);
            }
            MarkupTree::Value(Value::Data { data: None, .. }) => {
            }
            */
            _ => {}
        }

        match name.as_str() {
            "text" => {
                let mut text = Text::new(content.inner());

                for attr in attrs {
                    match attr.name().as_str() {
                        "size" => {
                            let pixels: iced::Pixels = attr.try_into()?;

                            // cosmic-text will panic if the text size is too small
                            if pixels.0 < 2.0 {
                                warn!("text size too small. clamping.");
                                text = text.size(2.0);
                            } else {
                                text = text.size(pixels);
                            }
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

                let button = Button::new(content).on_press_with(|| {
                    SnowcapMessage::from(Message::Button(node.element_id().clone()))
                });

                Ok(button.into())
            }

            "rule-horizontal" => {
                let height: Pixels = attrs
                    .get("height")
                    .ok_or_else(|| ConversionError::Missing("height".to_string()))?
                    .try_into()?;
                debug!("[Rule horizontal height={height:?}]");
                Ok(Rule::horizontal(height).into())
            }

            "rule-vertical" => {
                let width: Pixels = attrs
                    .get("width")
                    .ok_or_else(|| ConversionError::Missing("width".to_string()))?
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
                    SnowcapMessage::from(Message::Toggler {
                        id: node.element_id().clone(),
                        toggled,
                    })
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
            "themer" => {
                // Create an iced::Theme from the name in the "theme" attribute
                let theme: Theme = attrs
                    .get("theme")
                    .ok_or_else(|| ConversionError::Missing("theme".to_string()))?
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
            "pick-list" => match content.inner() {
                MarkupTree::Value(value) => {
                    if let Value::Array(values) = &*value.borrow() {
                        let current = if let Some(attr) = attrs.get("selected") {
                            Some((*attr.value()).to_string().clone())
                        } else {
                            None
                        };

                        let values: Vec<String> =
                            values.into_iter().map(|x| x.to_string()).collect();

                        let picklist = PickList::new(values, current, |selected| {
                            attrs.set("current_selection", Value::String(selected.clone()));
                            SnowcapMessage::from(Message::PickListSelected {
                                id: node.element_id().clone(),
                                selected,
                            })
                        });

                        Ok(picklist.into())
                    } else {
                        panic!("Expecting Value::Array")
                    }
                }
                _ => panic!("Expecting MarkupTree::Value"),
            },
            _ => {
                return Err(ConversionError::UnsupportedAttribute(format!(
                    "Unhandled element type {name}"
                )))
            }
        }
    }
}
