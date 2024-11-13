//! Tree Node containers for [`arbutus`] trees

use colored::Colorize;
use parking_lot::Mutex;
use std::string::ToString;

use std::sync::Arc;
use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

use strum::{EnumDiscriminants, EnumIter};
use xxhash_rust::xxh64::Xxh64;

use crate::module::data::ModuleData;
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
pub struct SnowcapNode {
    pub element_id: Option<String>,
    pub attrs: Attributes,
    content: Content,

    // Removing this from SnowcapNode, as it is not Send + Sync
    //pub widget: Option<DynamicWidget<M>>,
    state: State,
    module_data: Option<Box<dyn ModuleData>>,
}

impl Clone for SnowcapNode {
    fn clone(&self) -> Self {
        SnowcapNode {
            element_id: self.element_id.clone(),
            attrs: self.attrs.clone(),
            content: self.content.clone(),
            //widget: None,
            state: State::New,
            module_data: None,
        }
    }
}

impl std::hash::Hash for SnowcapNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //tracing::info!("Hashing SnowcapNode {}", self.data.to_string());
        self.element_id.hash(state);
        self.attrs.hash(state);
        self.content.hash(state);
    }
}

impl std::fmt::Display for SnowcapNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attr_display = if self.attrs.len() > 0 {
            format!("{:?}", self.attrs)
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

impl Default for SnowcapNode {
    fn default() -> Self {
        Self {
            content: Content::default(),
            element_id: None,
            attrs: Attributes::default(),
            //widget: None,
            state: State::New,
            module_data: None,
        }
    }
}

impl std::fmt::Debug for SnowcapNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(element_id) = &self.element_id {
            write!(f, "[{element_id:?}] ")?
        }
        write!(f, "{:?}", self.content)
    }
}

impl SnowcapNode {
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
    pub fn with_attrs(mut self, attrs: Attributes) -> Self {
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

    /// Set the Module Data for this node
    pub fn set_module_data(&mut self, data: Box<dyn ModuleData + 'static>) {
        self.module_data = Some(data);

        // Mark the node as dirty
        self.set_dirty(true);
    }

    /// Get a reference to the Module Data associated with this Node
    pub fn module_data(&self) -> Option<&Box<dyn ModuleData>> {
        self.module_data.as_ref()
    }
}

/// Deref into the inner [`Content`]
impl Deref for SnowcapNode {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}
