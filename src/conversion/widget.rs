use crate::attribute::{AttributeType, AttributeValue};
use crate::{NodeId, NodeRef};
use arbutus::Node as _;
use arbutus::NodeRef as _;
use iced::advanced::Widget;
use iced::widget::Text;
use iced::widget::{Button, PickList, Rule, Space, Themer, Toggler};
use tracing::warn;

use crate::attribute::Attributes;
use crate::data::DataType;
use crate::dynamic_widget::DynamicWidget;
use crate::error::ConversionError;
use crate::message::WidgetMessage;
use crate::node::SnowcapNodeData;
use crate::parser::Value;

pub struct SnowcapWidget<M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub node: NodeRef<M>,
}

impl<M> SnowcapWidget<M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)>,
{
    pub fn new(node: NodeRef<M>) -> Self {
        Self { node }
    }

    pub fn loading() -> DynamicWidget<M> {
        DynamicWidget::new(Text::new("Loading"))
    }

    pub fn missing() -> DynamicWidget<M> {
        DynamicWidget::new(Text::new("Missing"))
    }

    fn with_value<R, F>(node: &NodeRef<M>, f: F) -> Result<R, ConversionError>
    where
        F: FnOnce(&Value) -> Result<R, ConversionError>,
    {
        node.with_data(|node| match &node.data {
            SnowcapNodeData::Value(value) => f(value),
            _ => Err(ConversionError::InvalidType(
                "Expecting Value node in get_node_value".into(),
            )),
        })
    }

    pub fn from_tree_node(
        node_id: NodeId,
        name: String,
        element_id: Option<String>,
        attrs: Attributes,
        content: Option<NodeRef<M>>,
    ) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer>>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        // Handle any nodes with a value of Value::Data
        // as we can infer the widget type from the DataType
        // using .to_widget()

        if let Some(content) = content.clone() {
            match &content.node().data().data {
                SnowcapNodeData::Value(value) => match value {
                    Value::Data {
                        data: Some(data), ..
                    } => {
                        return DataType::to_widget(node_id, data.clone(), attrs.clone());
                    }
                    Value::Data { data: None, .. } => {
                        // No data is available, so provide a placeholder widget
                        // TODO: Some kind of spinner?
                        return Ok(Box::new(SnowcapWidget::loading()));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        match name.as_str() {
            "text" => {
                let content = content.ok_or(ConversionError::Missing("text content".into()))?;
                let text = Self::with_value(&content, |value| {
                    let mut text = Text::new(value);
                    let mut style = iced::widget::text::Style::default();

                    //TODO add wrapping

                    for attr in attrs {
                        (text, style) = match *attr {
                            AttributeValue::TextColor(color) => {
                                style.color = Some(color);
                                (text.color(color), style)
                            }
                            AttributeValue::Border(border) => todo!(),
                            AttributeValue::Shadow(shadow) => todo!(),
                            AttributeValue::HorizontalAlignment(horizontal) => {
                                (text.align_x(horizontal), style)
                            }
                            AttributeValue::VerticalAlignment(vertical) => {
                                (text.align_y(vertical), style)
                            }
                            AttributeValue::Padding(padding) => todo!(),
                            AttributeValue::WidthLength(length) => (text.width(length), style),
                            AttributeValue::WidthPixels(pixels) => (text.width(pixels), style),
                            AttributeValue::MaxWidth(pixels) => todo!(),
                            AttributeValue::HeightLength(length) => (text.height(length), style),
                            AttributeValue::HeightPixels(pixels) => (text.height(pixels), style),
                            AttributeValue::Background(background) => todo!(),
                            AttributeValue::Spacing(pixels) => todo!(),
                            AttributeValue::Size(pixels) => (text.size(pixels), style),
                            AttributeValue::CellSize(pixels) => todo!(),
                            AttributeValue::Clip(_) => todo!(),
                            AttributeValue::Toggled(_) => todo!(),
                            AttributeValue::Selected(_) => todo!(),
                            AttributeValue::Label(_) => todo!(),
                            _ => {
                                warn!("Unsupported Text attribute {:?}", attr);
                                (text, style)
                            }
                        };
                    }

                    //Ok(Box::new(text.style(move |_theme| style)))
                    Ok(Box::new(text))
                })?;

                Ok(text)
            }
            "space" => {
                let space = Space::new(1, 1);
                Ok(Box::new(space))
            }

            "button" => {
                let content = content.ok_or(ConversionError::Missing("button content".into()))?;
                let content = DynamicWidget::from_node(content)?.into_element();

                let button = Button::new(content).on_press_with(move || {
                    M::from((node_id, WidgetMessage::Button(element_id.clone())))
                });

                Ok(Box::new(button))
            }
            "rule-horizontal" => {
                //debug!("[Rule horizontal height={height:?}]");
                Ok(Box::new(Rule::horizontal(1)))
            }

            "rule-vertical" => Ok(Box::new(Rule::vertical(1))),

            "toggler" => {
                let is_toggled: bool = if let AttributeValue::Toggled(toggled) =
                    *attrs.clone().get(AttributeType::Toggled).unwrap()
                {
                    toggled
                } else {
                    false
                };

                let _attrs = attrs.clone();
                let mut toggler = Toggler::new(is_toggled).on_toggle(move |toggled| {
                    //_attrs.set("toggled", Value::Boolean(toggled));
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

                Ok(Box::new(toggler))
            }
            /*
            "themer" => {
                let content = content.ok_or(ConversionError::Missing("themer content".into()))?;
                // Create an iced::Theme from the name in the "theme" attribute
                let theme: Theme = attrs
                    .get("theme")
                    .ok_or_else(|| ConversionError::Missing("theme".to_string()))?
                    .try_into()?;

                let content = DynamicWidget::from_node(content)?.into_element();

                let themer = Themer::new(
                    move |old_theme| {
                        info!("Themer from {:?} to {:?}", old_theme, theme);
                        theme.clone()
                    },
                    content,
                );
                Ok(Box::new(themer))
            }
            */
            "pick-list" => {
                let content =
                    content.ok_or(ConversionError::Missing("pick-list content".into()))?;
                let picklist = Self::with_value(&content, |value| {
                    if let Value::Array(values) = value {
                        let current = if let Some(attr) = attrs.clone().get(AttributeType::Selected)
                        {
                            Some((*attr.value()).to_string().clone())
                        } else {
                            None
                        };

                        let values: Vec<String> =
                            values.into_iter().map(|x| x.to_string()).collect();

                        let attrs = attrs.clone();
                        let picklist = PickList::new(values, current, move |selected| {
                            //attrs.set("selected", Value::String(selected.clone()));
                            M::from((
                                node_id,
                                WidgetMessage::PickListSelected {
                                    id: element_id.clone(),
                                    selected,
                                },
                            ))
                        });

                        Ok(picklist)
                    } else {
                        panic!("Expecting Value::Array")
                    }
                })?;

                Ok(Box::new(picklist))
            }
            _ => {
                return Err(ConversionError::UnsupportedWidget(format!(
                    "Unhandled element type {name}"
                )))
            }
        }
    }
}
