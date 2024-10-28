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
    pub element_id: Option<String>,
    pub attrs: Option<Attributes>,
    pub data: SnowcapNodeData,
    pub widget: Option<Box<dyn iced::advanced::Widget<M, iced::Theme, iced::Renderer>>>,
    pub dirty: bool,
}

impl<M> Clone for SnowcapNode<M> {
    fn clone(&self) -> Self {
        SnowcapNode {
            element_id: self.element_id.clone(),
            attrs: self.attrs.clone(),
            data: self.data.clone(),
            widget: None,
            dirty: false,
        }
    }
}

impl<M> std::hash::Hash for SnowcapNode<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //tracing::info!("Hashing SnowcapNode {}", self.data.to_string());
        self.element_id.hash(state);
        self.attrs.hash(state);
        self.data.hash(state);
    }
}

impl<M> std::fmt::Display for SnowcapNode<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attr_display = if let Some(attrs) = &self.attrs {
            format!("{:?}", attrs)
        } else {
            "".into()
        };

        write!(
            f,
            "{} {}",
            self.data.to_string().cyan(),
            attr_display.green(),
        )
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
}

impl<M> Deref for SnowcapNode<M> {
    type Target = SnowcapNodeData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
