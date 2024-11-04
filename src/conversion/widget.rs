use crate::attribute::{AttributeKind, AttributeValue};
use crate::cache::WidgetContent;
use crate::util::ElementWrapper;
//use crate::util::ElementWrapper;
use crate::NodeId;
use iced::widget::{Button, PickList, Rule, Space, Themer, Toggler};
use iced::widget::{Image, Svg, Text};
use tracing::warn;

use crate::attribute::Attributes;
use crate::dynamic_widget::DynamicWidget;
use crate::error::ConversionError;
use crate::message::WidgetMessage;

pub struct SnowcapWidget;

impl SnowcapWidget {
    pub fn loading<'a, M>() -> DynamicWidget<M> {
        DynamicWidget::default()
            .with_widget(Text::new("Loading"))
            .with_node_id(8989898)
    }

    pub fn new<'a, M>(
        node_id: NodeId,
        name: String,
        element_id: Option<String>,
        attrs: Attributes,
        content: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        match name.as_str() {
            "image" => match content {
                WidgetContent::Module(module) => {
                    println!("Image has Module content {module}");
                    Ok(DynamicWidget::default().with_widget(Text::new("loading")))
                }
                WidgetContent::Image(handle) => {
                    Ok(DynamicWidget::default().with_widget(Image::new(handle)))
                }
                _ => Err(ConversionError::InvalidType(format!(
                    "Image expecting WidgetContent::Image {}:{}",
                    file!(),
                    line!()
                ))),
            },
            "svg" => match content {
                WidgetContent::Module(_module) => {
                    Ok(DynamicWidget::default().with_widget(Text::new("loading")))
                }
                WidgetContent::Svg(handle) => {
                    let svg = Svg::new(handle);
                    Ok(DynamicWidget::default().with_widget(svg))
                }
                _ => Err(ConversionError::InvalidType(format!(
                    "Image expecting WidgetContent::Image {}:{}",
                    file!(),
                    line!()
                ))),
            },
            "markdown" => match content {
                WidgetContent::Module(_module) => {
                    Ok(DynamicWidget::default().with_widget(Text::new("loading")))
                }
                //WidgetContent::Markdown(items) => {
                WidgetContent::Text(text) => {
                    let items: Vec<iced::widget::markdown::Item> =
                        iced::widget::markdown::parse(&text).collect();
                    let style =
                        iced::widget::markdown::Style::from_palette(iced::Theme::Light.palette());

                    let settings = iced::widget::markdown::Settings::default();

                    let markdown = iced::widget::markdown(&items, settings, style)
                        .map(move |url| M::from((node_id, WidgetMessage::Markdown(url))));
                    let wrapped = ElementWrapper::<M>::new(markdown);
                    Ok(DynamicWidget::default().with_widget(wrapped))
                }
                _ => Err(ConversionError::InvalidType(format!(
                    "Markdown expecting WidgetContent::Markdown {}:{}",
                    file!(),
                    line!()
                ))),
            },

            /*
            "qr-code" => {
                if let WidgetContent::Value(value) = content {
                    let data = value
                        .dynamic()?
                        .clone()
                        .ok_or(ConversionError::Missing("qr data".into()))?;

                    if let DataType::QrCode(qr_data) = &*data {
                        let mut qr = QRCode::new(qr_data.clone());

                        for attr in attrs {
                            qr = match *attr {
                                crate::attribute::AttributeValue::CellSize(pixels) => {
                                    qr.cell_size(pixels)
                                }
                                _ => {
                                    warn!("Unsupported QRCode attribute {:?}", attr);
                                    qr
                                }
                            };
                        }

                        Ok(DynamicWidget::default().with_widget(qr))
                    } else {
                        Ok(SnowcapWidget::loading())
                    }
                } else {
                    error!("QrCode got {content:#?}");
                    Err(ConversionError::InvalidType(
                        "QrCode expecting WidgetContent::Value".into(),
                    ))
                }
            }
            */
            "text" => {
                let mut text = if let WidgetContent::Value(value) = content {
                    Text::new(value.inner())
                } else if let WidgetContent::Text(value) = content {
                    Text::new(value)
                } else {
                    Text::new("X")
                };

                let mut style = iced::widget::text::Style::default();

                //TODO add shaping, font

                for attr in attrs {
                    (text, style) = match *attr {
                        AttributeValue::TextColor(color) => {
                            style.color = Some(color);
                            (text.color(color), style)
                        }
                        AttributeValue::HorizontalAlignment(horizontal) => {
                            (text.align_x(horizontal), style)
                        }
                        AttributeValue::VerticalAlignment(vertical) => {
                            (text.align_y(vertical), style)
                        }
                        AttributeValue::WidthLength(length) => (text.width(length), style),
                        AttributeValue::WidthPixels(pixels) => (text.width(pixels), style),
                        AttributeValue::HeightLength(length) => (text.height(length), style),
                        AttributeValue::HeightPixels(pixels) => (text.height(pixels), style),
                        AttributeValue::Size(pixels) => (text.size(pixels), style),
                        AttributeValue::Wrapping(wrapping) => (text.wrapping(wrapping), style),
                        AttributeValue::Shaping(shaping) => (text.shaping(shaping), style),
                        _ => {
                            warn!("Unsupported Text attribute {:?}", attr);
                            (text, style)
                        }
                    };
                }

                //Ok(Box::new(text.style(move |_theme| style)))
                //Ok(Box::new(text))
                Ok(DynamicWidget::default().with_widget(text))
            }
            "space" => {
                let space = Space::new(1, 1);
                //Ok(Box::new(space))
                Ok(DynamicWidget::default().with_widget(space))
            }

            "button" => {
                let button = Button::new(content).on_press_with(move || {
                    M::from((node_id, WidgetMessage::Button(element_id.clone())))
                });

                Ok(DynamicWidget::default().with_widget(button))
            }
            "rule-horizontal" => Ok(DynamicWidget::default().with_widget(Rule::horizontal(1))),
            "rule-vertical" => Ok(DynamicWidget::default().with_widget(Rule::vertical(1))),

            "toggler" => {
                let is_toggled: bool = if let Some(AttributeValue::Toggled(toggled)) =
                    attrs.clone().get(AttributeKind::Toggled)?
                {
                    toggled
                } else {
                    false
                };

                let _attrs = attrs.clone();
                let mut toggler = Toggler::new(is_toggled).on_toggle(move |toggled| {
                    _attrs.set(AttributeValue::Toggled(toggled)).unwrap();
                    M::from((
                        node_id,
                        WidgetMessage::Toggler {
                            id: element_id.clone(),
                            toggled,
                        },
                    ))
                });

                for attr in attrs {
                    toggler = match (*attr).clone() {
                        AttributeValue::Size(pixels) => toggler.size(pixels),
                        AttributeValue::Label(label) => toggler.label(label),
                        AttributeValue::Toggled(_) => toggler,
                        _ => todo!(),
                    };
                }

                Ok(DynamicWidget::default().with_widget(toggler))
            }

            "themer" => {
                let theme =
                    if let Some(AttributeValue::Theme(theme)) = attrs.get(AttributeKind::Theme)? {
                        Some(theme)
                    } else {
                        None
                    };

                let themer = Themer::new(
                    move |old_theme| {
                        tracing::info!("Themer from {:?} to {:?}", old_theme, theme);
                        theme.as_ref().unwrap().clone()
                    },
                    content,
                );
                Ok(DynamicWidget::default().with_widget(themer))
            }
            "pick-list" => {
                if let WidgetContent::Value(value) = content {
                    let current = if let Some(AttributeValue::Selected(selected)) =
                        attrs.get(AttributeKind::Selected)?
                    {
                        Some(selected)
                    } else {
                        None
                    };

                    let values: Vec<String> =
                        value.array()?.into_iter().map(|x| x.to_string()).collect();

                    let _attrs = attrs.clone();
                    let picklist = PickList::new(values, current, move |selected| {
                        _attrs
                            .set(AttributeValue::Selected(selected.clone()))
                            .unwrap();

                        M::from((
                            node_id,
                            WidgetMessage::PickListSelected {
                                id: element_id.clone(),
                                selected,
                            },
                        ))
                    });

                    Ok(DynamicWidget::default().with_widget(picklist))
                } else {
                    Err(ConversionError::InvalidType("expecting value array".into()))
                }
            }
            _ => {
                return Err(ConversionError::UnsupportedWidget(format!(
                    "Unhandled element type {name}"
                )))
            }
        }
    }
}
