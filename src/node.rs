//! Tree Node containers for [`arbutus`] trees

use colored::Colorize;
use std::string::ToString;

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use strum::{EnumDiscriminants, EnumIter};
use xxhash_rust::xxh64::Xxh64;

use crate::dynamic_widget::DynamicWidget;
use crate::parser::module::Module;
use crate::{attribute::Attributes, Value};

#[derive(Debug, Hash, Clone, EnumDiscriminants, strum::Display)]
#[strum_discriminants(derive(EnumIter, strum::Display, Hash, PartialOrd, Ord))]
#[strum_discriminants(name(SnowcapNodeKind))]
pub enum Content {
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
    #[strum(to_string = "Module {0}")]
    Module(Module),
}

impl Content {
    pub fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Node newly introduced to tree
    New,
    /// Node is dirty, needs to be updated
    Dirty,
    /// Node is clean
    Clean,
}

/// Snowcap tree node. This is the node type used in the [`Arbutus`](https://github.com/boondocklabs/arbutus) tree
/// which is built by the markup parser
pub struct SnowcapNode<M>
where
    M: 'static,
{
    pub element_id: Option<String>,
    pub attrs: Option<Attributes>,
    content: Content,
    pub widget: Option<DynamicWidget<M>>,
    state: State,
}

impl<M> Clone for SnowcapNode<M> {
    fn clone(&self) -> Self {
        SnowcapNode {
            element_id: self.element_id.clone(),
            attrs: self.attrs.clone(),
            content: self.content.clone(),
            widget: None,
            state: State::New,
        }
    }
}

impl<M> std::hash::Hash for SnowcapNode<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //tracing::info!("Hashing SnowcapNode {}", self.data.to_string());
        self.element_id.hash(state);
        self.attrs.hash(state);
        self.content.hash(state);
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
            self.content.to_string().cyan(),
            attr_display.green(),
        )
    }
}

impl<M> Default for SnowcapNode<M> {
    fn default() -> Self {
        Self {
            content: Content::default(),
            element_id: None,
            attrs: None,
            widget: None,
            state: State::New,
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
        write!(f, "{:?}", self.content)
    }
}

impl<M> SnowcapNode<M> {
    pub fn new(content: Content) -> Self {
        Self::default().with_content(content)
    }

    /// Add Content to this node
    pub fn with_content(mut self, content: Content) -> Self {
        self.content = content;
        self
    }

    /// Add an element ID to this node
    pub fn with_element_id(mut self, id: Option<String>) -> Self {
        self.element_id = id;
        self
    }

    /// Add attributes to this node
    pub fn with_attrs(mut self, attrs: Option<Attributes>) -> Self {
        self.attrs = attrs;
        self
    }

    /// Set the dirty state of this node
    pub fn set_dirty(&mut self, dirty: bool) {
        match dirty {
            true => self.set_state(State::Dirty),
            false => self.set_state(State::Clean),
        }
    }

    /// Return true if the node state is State::Dirty
    pub fn is_dirty(&self) -> bool {
        self.state == State::Dirty
    }

    /// Return true if the node state is State::New
    pub fn is_new(&self) -> bool {
        self.state == State::New
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        //debug!("{} {:?} -> {:?}", self, self.state, state);
        self.state = state
    }

    /// Get a reference to the node content
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Get a mutable reference to the node content
    pub fn content_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

/// Deref into the inner [`Content`]
impl<M> Deref for SnowcapNode<M> {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}
