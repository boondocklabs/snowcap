use std::ops::Deref;

use iced::{widget::Text, Element};

use crate::{attribute::Attributes, widget::WidgetWrap, ConversionError, Value};

#[derive(Debug)]
pub enum SnowcapNodeData {
    None,
    Root,
    Container,
    Widget(String),
    Row,
    Column,
    Stack,
    Value(Value),
}

impl Default for SnowcapNodeData {
    fn default() -> Self {
        Self::None
    }
}

pub struct SnowcapNode<M> {
    pub data: SnowcapNodeData,
    pub element_id: Option<String>,
    pub attrs: Option<Attributes>,
    pub widget: Option<WidgetWrap<M>>,
    pub dirty: bool,
}

impl<M> std::fmt::Display for SnowcapNode<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<M> Default for SnowcapNode<M> {
    fn default() -> Self {
        Self {
            data: SnowcapNodeData::default(),
            element_id: None,
            attrs: None,
            widget: None,
            dirty: false,
        }
    }
}

impl<M> std::fmt::Debug for SnowcapNode<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(element_id) = &self.element_id {
            write!(f, "[{element_id:?}] ")?
        }
        write!(f, "{:?}", self.data)
    }
}

impl<M> SnowcapNode<M> {
    pub fn new(data: SnowcapNodeData) -> Self {
        SnowcapNode {
            data,
            element_id: None,
            attrs: None,
            widget: None,
            dirty: false,
        }
    }

    pub fn with_element_id(mut self, id: Option<String>) -> Self {
        self.element_id = id;
        self
    }

    pub fn with_attrs(mut self, attrs: Option<Attributes>) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn as_element(&self) -> Result<Element<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + 'static,
    {
        if let Some(widget) = self.widget.as_ref() {
            Ok(Element::new(widget.widget()))
        } else {
            Ok(Element::new(Text::new(format!(
                "as_element(): No widget in node {self:#?}"
            ))))
        }
    }
}

impl<M> Deref for SnowcapNode<M> {
    type Target = SnowcapNodeData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
