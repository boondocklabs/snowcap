use colored::Colorize;
use std::string::ToString;

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use strum::{EnumDiscriminants, EnumIter};
use xxhash_rust::xxh64::Xxh64;

use crate::{attribute::Attributes, Value};

#[derive(Debug, Hash, Clone, EnumDiscriminants, strum::Display)]
#[strum_discriminants(derive(EnumIter, strum::Display, Hash, PartialOrd, Ord))]
#[strum_discriminants(name(SnowcapNodeKind))]
pub enum SnowcapNodeData {
    None,
    Root,
    Container,
    #[strum(to_string = "Widget: {0}")]
    Widget(String),
    Row,
    Column,
    Stack,
    #[strum(to_string = "Value: {0}")]
    Value(Value),
}

impl SnowcapNodeData {
    pub fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.hash(&mut hasher);
        hasher.finish()
    }
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
    pub widget: Option<Box<dyn iced::advanced::Widget<M, iced::Theme, iced::Renderer>>>,
    //pub widget: Option<WidgetWrap<M>>,
    pub dirty: bool,
}

impl<M> std::fmt::Display for SnowcapNode<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.to_string().cyan())
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

    pub fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.data.hash(&mut hasher);
        self.element_id.hash(&mut hasher);
        self.attrs.hash(&mut hasher);
        hasher.finish()
    }
}

impl<M> Deref for SnowcapNode<M> {
    type Target = SnowcapNodeData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
