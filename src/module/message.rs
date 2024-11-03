use std::any::Any;
use std::sync::Arc;

use colored::Colorize;

use crate::Error;
use crate::Message;

use super::{event::ModuleEvent, HandleId};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Topic(pub &'static str);

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.bright_green())
    }
}

#[derive(Clone, Debug)]
pub enum TopicMessage {
    Trigger,
    String(String),
}

impl std::fmt::Display for TopicMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).bright_cyan())
    }
}

#[derive(Clone, Debug)]
pub struct PublishMessage {
    pub topic: Topic,
    pub message: TopicMessage,
}

impl std::fmt::Display for PublishMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[topic={} message={}]",
            self.topic.to_string().magenta(),
            self.message.to_string().green()
        )
    }
}

#[derive(Default, Debug, Clone)]
pub enum ModuleMessage {
    #[default]
    None,
    Debug(&'static str),
    Error(Arc<crate::Error>),
    Event(Arc<Box<dyn Any + Send + Sync>>),

    /// Module requesting a subscription to a channel
    Subscribe(Topic),

    /// Publish a message to a channel
    Publish(PublishMessage),

    /// A published message being sent to a module
    Published(PublishMessage),
}

/// A container for [`ModuleMessage`] containing additional decorations
/// for dynamic dispatch of messages to module instances.
#[derive(Debug, Clone)]
pub struct ModuleMessageContainer {
    handle_id: HandleId,
    message: ModuleMessage,
}

impl ModuleMessageContainer {
    /// Create a new ModuleMessage with the specified module HandleId and inner message kind
    pub fn new(handle_id: HandleId, kind: ModuleMessage) -> Self {
        Self {
            handle_id,
            message: kind,
        }
    }

    /// Get the module handle ID associated with this message
    pub fn handle_id(&self) -> HandleId {
        self.handle_id
    }

    /// Get a reference to the inner [`ModuleMessage`].
    ///
    /// To take ownership of the inner kind, use [`ModuleMessage::take_kind()`]
    pub fn message(&self) -> &ModuleMessage {
        &self.message
    }

    /// Get a mutable reference to the inner [`ModuleMessage`]
    pub fn message_mut(&mut self) -> &mut ModuleMessage {
        &mut self.message
    }

    /// Take the inner [`ModuleMessage`] out of this [`ModuleMessage`],
    /// leaving the default in its place.
    pub fn take_message(&mut self) -> ModuleMessage {
        std::mem::take(&mut self.message)
    }
}

impl<T: ModuleEvent + 'static> From<(HandleId, T)> for ModuleMessageContainer {
    fn from(value: (HandleId, T)) -> Self {
        ModuleMessageContainer {
            handle_id: value.0,
            message: ModuleMessage::Event(Arc::new(Box::new(value.1))),
        }
    }
}

/// Implement from any T which implements [`ModuleEvent`] on Message.
/// This will wrap the [`ModuleEvent`] in an Arc, and return a new
/// Message::ModuleEvent message containing an Arc<dyn ModuleEvent>
impl<AppMessage> From<ModuleMessageContainer> for Message<AppMessage> {
    fn from(value: ModuleMessageContainer) -> Self {
        Message::Module(value)
    }
}

/// Convert a tuple of (HandleId, crate::Error) into a ModuleMessage
impl From<(HandleId, Error)> for ModuleMessageContainer {
    fn from(value: (HandleId, Error)) -> Self {
        ModuleMessageContainer::new(value.0, ModuleMessage::Error(Arc::new(value.1)))
    }
}

impl From<Error> for ModuleMessage {
    fn from(err: Error) -> Self {
        ModuleMessage::Error(Arc::new(err))
    }
}
