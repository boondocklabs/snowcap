use crate::attribute::{AttributeKind, AttributeValue};
use crate::data::DataType;
use crate::parser::value::ValueKind;
use crate::tree_util::ChildData;
use crate::util::ElementWrapper;
use crate::NodeId;
use iced::widget::{Button, PickList, QRCode, Rule, Space, Svg, Themer, Toggler};
use iced::widget::{Image, Text};
use tracing::warn;

use crate::attribute::Attributes;
use crate::dynamic_widget::DynamicWidget;
use crate::error::ConversionError;
use crate::message::WidgetMessage;

pub struct SnowcapWidget;

impl SnowcapWidget {
    pub fn loading<M>() -> DynamicWidget<'static, M> {
        DynamicWidget::default()
            .with_widget(Text::new("Loading"))
            .with_node_id(8989898)
    }

    pub fn new<'a, M>(
        node_id: NodeId,
        name: String,
        element_id: Option<String>,
        attrs: Attributes,
        content: Option<Vec<ChildData<'static, M>>>,
    ) -> Result<DynamicWidget<'a, M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'a,
    {
        match name.as_str() {
            "image" => {
                let mut content =
                    content.ok_or(ConversionError::Missing("image content".into()))?;
                let content = content
                    .pop()
                    .ok_or(ConversionError::Missing("image content".into()))?;

                if let ChildData::Value(value) = content {
                    match value.inner() {
                        ValueKind::String(_) => todo!(),
                        ValueKind::Float(_) => todo!(),
                        ValueKind::Integer(_) => todo!(),
                        ValueKind::Boolean(_) => todo!(),
                        ValueKind::Array(_vec) => todo!(),
                        ValueKind::Dynamic { data, provider: _ } => {
                            if let Some(data) = data {
                                if let DataType::Image(handle) = &**data {
                                    Ok(DynamicWidget::default().with_widget(Image::new(handle)))
                                } else {
                                    Ok(SnowcapWidget::loading())
                                }
                            } else {
                                Ok(SnowcapWidget::loading())
                            }
                        }
                    }
                } else {
                    Err(ConversionError::InvalidType(
                        "Image expecting ChildData::Value".into(),
                    ))
                }
            }
            "svg" => {
                let mut content = content.ok_or(ConversionError::Missing("svg content".into()))?;
                let content = content
                    .pop()
                    .ok_or(ConversionError::Missing("svg content".into()))?;

                if let ChildData::Value(value) = content {
                    match value.inner() {
                        ValueKind::String(_) => todo!(),
                        ValueKind::Float(_) => todo!(),
                        ValueKind::Integer(_) => todo!(),
                        ValueKind::Boolean(_) => todo!(),
                        ValueKind::Array(_vec) => todo!(),
                        ValueKind::Dynamic { data, provider: _ } => {
                            if let Some(data) = data {
                                if let DataType::Svg(handle) = &**data {
                                    Ok(DynamicWidget::default()
                                        .with_widget(Svg::new(handle.clone())))
                                } else {
                                    Ok(SnowcapWidget::loading())
                                }
                            } else {
                                Ok(SnowcapWidget::loading())
                            }
                        }
                    }
                } else {
                    Err(ConversionError::InvalidType(
                        "Image expecting ChildData::Value".into(),
                    ))
                }
            }
            "markdown" => {
                let mut content =
                    content.ok_or(ConversionError::Missing("markdown content".into()))?;
                let content = content
                    .pop()
                    .ok_or(ConversionError::Missing("markdown content".into()))?;

                if let ChildData::Value(value) = content {
                    match value.inner() {
                        ValueKind::Dynamic { data, provider: _ } => {
                            if let Some(data) = data {
                                if let DataType::Markdown(markdown_items) = &**data {
                                    let markdown: iced::Element<'static, M> =
                                        iced::widget::markdown(
                                            markdown_items.into_iter(),
                                            iced::widget::markdown::Settings::default(),
                                            iced::widget::markdown::Style::from_palette(
                                                iced::Theme::default().palette(),
                                            ),
                                        )
                                        .map(move |url| {
                                            M::from((node_id, WidgetMessage::Markdown(url)))
                                        });

                                    Ok(DynamicWidget::default()
                                        .with_widget(ElementWrapper::new(markdown)))
                                } else {
                                    Ok(SnowcapWidget::loading())
                                }
                            } else {
                                Ok(SnowcapWidget::loading())
                            }
                        }
                        _ => Err(ConversionError::InvalidType(
                            "unexpected markdown {value:?}".into(),
                        )),
                    }
                } else {
                    Err(ConversionError::InvalidType(
                        "unexpected markdown {content:?} expecting ChildData::Value".into(),
                    ))
                }
            }
            "qr-code" => {
                let mut content =
                    content.ok_or(ConversionError::Missing("qr-code content".into()))?;
                let content = content
                    .pop()
                    .ok_or(ConversionError::Missing("qr-code content".into()))?;

                if let ChildData::Value(value) = content {
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
                    Err(ConversionError::InvalidType(
                        "expecting ChildData::Value".into(),
                    ))
                }
            }
            "text" => {
                let mut content = content.ok_or(ConversionError::Missing("text content".into()))?;
                let mut text = if let Some(ChildData::Value(value)) = content.pop() {
                    Text::new(value.inner())
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
                if let Some(mut content) = content {
                    let button = Button::new(content.pop().unwrap()).on_press_with(move || {
                        M::from((node_id, WidgetMessage::Button(element_id.clone())))
                    });

                    Ok(DynamicWidget::default().with_widget(button))
                } else {
                    Err(ConversionError::Missing("button content".into()))
                }
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
                let mut content =
                    content.ok_or(ConversionError::Missing("themer content".into()))?;

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
                    content.pop().unwrap(),
                );
                Ok(DynamicWidget::default().with_widget(themer))
            }
            "pick-list" => {
                let mut content =
                    content.ok_or(ConversionError::Missing("pick-list content".into()))?;

                if let Some(ChildData::Value(value)) = content.pop() {
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
