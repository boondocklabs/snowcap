use std::any::Any;
use std::sync::Arc;

use crate::Error;
use crate::Message;

use super::{event::ModuleEvent, HandleId};

#[derive(Default, Debug, Clone)]
pub enum ModuleMessageKind {
    #[default]
    None,
    Debug(&'static str),
    Error(Arc<crate::Error>),
    Event(Arc<Box<dyn Any + Send + Sync>>),
}

#[derive(Debug, Clone)]
pub struct ModuleMessage {
    handle_id: HandleId,
    kind: ModuleMessageKind,
}

impl ModuleMessage {
    /// Create a new ModuleMessage with the specified module HandleId and inner message kind
    pub fn new(handle_id: HandleId, kind: ModuleMessageKind) -> Self {
        Self { handle_id, kind }
    }

    /// Get the module handle ID associated with this message
    pub fn handle_id(&self) -> HandleId {
        self.handle_id
    }

    /// Get a reference to the inner [`ModuleMessageKind`].
    ///
    /// To take ownership of the inner kind, use [`ModuleMessage::take_kind()`]
    pub fn kind(&self) -> &ModuleMessageKind {
        &self.kind
    }

    /// Take the inner [`ModuleMessageKind`] out of this [`ModuleMessage`],
    /// leaving the default in its place.
    pub fn take_kind(&mut self) -> ModuleMessageKind {
        std::mem::take(&mut self.kind)
    }
}

impl<T: ModuleEvent + 'static> From<(HandleId, T)> for ModuleMessage {
    fn from(value: (HandleId, T)) -> Self {
        ModuleMessage {
            handle_id: value.0,
            kind: ModuleMessageKind::Event(Arc::new(Box::new(value.1))),
        }
    }
}

/// Implement from any T which implements [`ModuleEvent`] on Message.
/// This will wrap the [`ModuleEvent`] in an Arc, and return a new
/// Message::ModuleEvent message containing an Arc<dyn ModuleEvent>
impl<AppMessage> From<ModuleMessage> for Message<AppMessage> {
    fn from(value: ModuleMessage) -> Self {
        Message::Module(value)
    }
}

/// Convert a tuple of (HandleId, crate::Error) into a ModuleMessage
impl From<(HandleId, Error)> for ModuleMessage {
    fn from(value: (HandleId, Error)) -> Self {
        ModuleMessage::new(value.0, ModuleMessageKind::Error(Arc::new(value.1)))
    }
}
