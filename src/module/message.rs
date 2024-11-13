use crate::Message;

use super::ModuleHandleId;

/// A container for [`ModuleMessage`] containing additional decorations
/// for dynamic dispatch of messages to module instances.
#[derive(Debug, Clone)]
pub struct ModuleMessage {
    handle_id: ModuleHandleId,
    message: Message,
}

impl ModuleMessage {
    /// Create a new ModuleMessage with the specified module HandleId and inner message kind
    pub fn new(handle_id: ModuleHandleId, message: Message) -> Self {
        Self { handle_id, message }
    }

    /// Get the module handle ID associated with this message
    pub fn handle_id(&self) -> ModuleHandleId {
        self.handle_id
    }

    /// Get a reference to the inner [`Message`].
    ///
    /// To take ownership of the inner kind, use [`ModuleMessage::into_inner()`]
    pub fn inner(&self) -> &Message {
        &self.message
    }

    /// Return the inner ModuleMessageData
    pub fn into_inner(self) -> Message {
        self.message
    }
}
