use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use iced::advanced::Widget;
use iced::widget::{Button, PickList, Rule, Space, Themer, Toggler};
use iced::{widget::Text, Element};
use iced::{Length, Pixels, Theme};
use tracing::{debug, info, warn};

use crate::attribute::Attributes;
use crate::data::DataType;
use crate::error::ConversionError;
use crate::message::WidgetMessage;
use crate::parser::Value;
use crate::tree::node::TreeNode;
use crate::widget::WidgetRefInner;
use crate::MarkupTreeNode;

pub struct SnowcapWidget<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub node: MarkupTreeNode<'a, M>,
    pub widget: WidgetRefInner<'a, M>,
}

/*
impl<'a, M> Clone for SnowcapWidget<'a, M>
where
    M: Clone + std::fmt::Debug + 'a,
{
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            widget: self.widget.clone(),
        }
    }
}
*/

impl<'a, M> SnowcapWidget<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn new(
        node: MarkupTreeNode<'a, M>,
        widget: Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'a>,
    ) -> Self {
        Self { node, widget }
    }

    pub fn loading() -> Box<dyn Widget<M, iced::Theme, iced::Renderer>> {
        Box::new(Text::new("Loading"))
    }

    pub fn from_tree_node(
        name: String,
        attrs: Attributes,
        content: TreeNode<'a, M>,
    ) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'a>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        // Handle any nodes with a value of Value::Data
        // as we can infer the widget type from the DataType
        // using .to_widget()

        match &*content.inner.borrow() {
            MarkupTreeNode::Value(value) => match &*(*value).borrow() {
                Value::Data {
                    data: Some(data), ..
                } => {
                    return DataType::to_widget(data.clone(), attrs.clone());
                }
                Value::Data { data: None, .. } => {
                    // No data is available, so provide a placeholder widget
                    // TODO: Some kind of spinner?
                    return Ok(SnowcapWidget::loading());
                }
                _ => {}
            },
            _ => {}
        }

        match name.as_str() {
            "text" => {
                let mut text = Text::new(content.inner_ref().clone());

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

                Ok(Box::new(text))
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

                Ok(Box::new(space))
            }

            "button" => {
                let element_id = content.inner_ref().get_element_id().clone();
                let content: Element<M> = content.into_element()?;

                let button = Button::new(content)
                    .on_press_with(move || M::from(WidgetMessage::Button(element_id.clone())));

                Ok(Box::new(button))
            }
            "rule-horizontal" => {
                let height: Pixels = attrs
                    .get("height")
                    .ok_or_else(|| ConversionError::Missing("height".to_string()))?
                    .try_into()?;
                debug!("[Rule horizontal height={height:?}]");
                Ok(Box::new(Rule::horizontal(height)))
            }

            "rule-vertical" => {
                let width: Pixels = attrs
                    .get("width")
                    .ok_or_else(|| ConversionError::Missing("width".to_string()))?
                    .try_into()?;
                Ok(Box::new(Rule::vertical(width)))
            }

            "toggler" => {
                let is_toggled: bool = if let Some(width) = attrs.get("toggled") {
                    width.try_into()?
                } else {
                    false
                };

                let element_id = content.inner.borrow().get_element_id().clone();
                let _attrs = attrs.clone();
                let mut toggler = Toggler::new(is_toggled).on_toggle(move |toggled| {
                    _attrs.set("toggled", Value::Boolean(toggled));
                    M::from(WidgetMessage::Toggler {
                        id: element_id.clone(),
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

                Ok(Box::new(toggler))
            }
            "themer" => {
                // Create an iced::Theme from the name in the "theme" attribute
                let theme: Theme = attrs
                    .get("theme")
                    .ok_or_else(|| ConversionError::Missing("theme".to_string()))?
                    .try_into()?;

                let content: Element<M, Theme> = content.clone().into_element()?;

                let themer = Themer::new(
                    move |old_theme| {
                        info!("Themer from {:?} to {:?}", old_theme, theme);
                        theme.clone()
                    },
                    content,
                );
                Ok(Box::new(themer))
            }
            "pick-list" => match &*content.inner.borrow() {
                MarkupTreeNode::Value(value) => {
                    if let Value::Array(values) = &*(**value).borrow() {
                        let current = if let Some(attr) = attrs.get("selected") {
                            Some((*attr.value()).to_string().clone())
                        } else {
                            None
                        };

                        let values: Vec<String> =
                            values.into_iter().map(|x| x.to_string()).collect();

                        let element_id = content.inner.borrow().get_element_id().clone();
                        let attrs = attrs.clone();
                        let picklist = PickList::new(values, current, move |selected| {
                            attrs.set("current_selection", Value::String(selected.clone()));
                            M::from(WidgetMessage::PickListSelected {
                                id: element_id.clone(),
                                selected,
                            })
                        });

                        Ok(Box::new(picklist))
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

/// Implementation of iced Widget trait on SnowcapWidget
impl<'a, SnowcapMessage> Widget<SnowcapMessage, iced::Theme, iced::Renderer>
    for SnowcapWidget<'a, SnowcapMessage>
where
    SnowcapMessage: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget.diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget.size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget.size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget.operate(tree, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, SnowcapMessage>,
        viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        self.widget.on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.widget
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.widget
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<SnowcapMessage, iced::Theme, iced::Renderer>> {
        self.widget.overlay(tree, layout, renderer, translation)
    }
}
