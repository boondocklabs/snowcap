use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use xxhash_rust::xxh64::Xxh64;

use crate::{attribute::Attributes, NodeId, Value};

#[derive(Debug, Hash)]
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

pub enum SnowcapNodeComparison {
    Equal,
    DataDiffer,
    AttributeDiffer,
    BothDiffer,
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

    /// Compare two SnowcapNode instances, returning a SnowcapNodeComparison enum
    /// describing if the nodes are the same or if the data, attributes, or both are different
    pub fn compare(&self, other: &Self) -> SnowcapNodeComparison {
        let data_equal = self.data.xxhash() == other.xxhash();

        let attrs_equal = self
            .attrs
            .as_ref()
            .zip(other.attrs.as_ref()) // Combine the two Options if both are Some
            .map_or(
                self.attrs.is_none() && other.attrs.is_none(),
                |(ours, theirs)| ours.xxhash() == theirs.xxhash(),
            );

        if data_equal && attrs_equal {
            SnowcapNodeComparison::Equal
        } else if data_equal && !attrs_equal {
            SnowcapNodeComparison::DataDiffer
        } else if !data_equal && attrs_equal {
            SnowcapNodeComparison::AttributeDiffer
        } else {
            SnowcapNodeComparison::BothDiffer
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

    /*
    pub fn as_element(&self) -> Result<Element<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + 'static,
    {
        if let Some(widget) = self.widget.as_ref() {
            Ok(Element::new(widget))
        } else {
            Ok(Element::new(Text::new(format!(
                "as_element(): No widget in node {self:#?}"
            ))))
        }
    }
    */
}

impl<M> Deref for SnowcapNode<M> {
    type Target = SnowcapNodeData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
