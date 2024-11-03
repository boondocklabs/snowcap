//! Messages for internal snowcap, module, and application communications

use std::{hash::Hash, path::PathBuf};

use iced::widget::markdown::Url;
use strum::{EnumDiscriminants, EnumIter};

use crate::{module::message::ModuleMessageContainer, parser::ElementId, NodeId};

/// Represents a message that can be passed within the application.
/// This enum encapsulates both application-specific messages and other events.
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum Message<AppMessage> {
    // Provide a default empty state, to allow std::mem::take()
    // to take ownership of a variant
    Empty,

    /// A variant that contains an application-specific message.
    ///
    /// # Type Parameters
    ///
    /// * `AppMessage` - The type of the application-specific message.
    App(AppMessage),

    Widget {
        node_id: NodeId,
        message: WidgetMessage,
    },

    Event(Event),

    Module(ModuleMessageContainer),

    Command(Command),
}

#[derive(Debug, Clone)]
pub enum WidgetMessage {
    /// A message variant for handling markdown-related events.
    ///
    /// This is used when an event related to markdown content
    /// occurs within the application.
    Markdown(Url),

    /// A variant for handling button events.
    Button(Option<ElementId>),

    /// A message variant for handling toggler events.
    Toggler {
        id: Option<ElementId>,
        toggled: bool,
    },

    /// A pick list was selected
    PickListSelected {
        id: Option<ElementId>,
        selected: String,
    },
}

impl<AppMessage> Default for Message<AppMessage> {
    fn default() -> Self {
        Message::Empty
    }
}

impl<AppMessage> From<Event> for Message<AppMessage> {
    fn from(event: Event) -> Self {
        Message::Event(event)
    }
}

impl<AppMessage> From<Command> for Message<AppMessage> {
    fn from(command: Command) -> Self {
        Message::Command(command)
    }
}

impl<AppMessage> From<(NodeId, WidgetMessage)> for Message<AppMessage> {
    fn from(widget_message: (NodeId, WidgetMessage)) -> Self {
        Message::Widget {
            node_id: widget_message.0,
            message: widget_message.1,
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(EventKind))]
pub enum Event {
    Empty,

    // Request a file be added to the watcher
    #[cfg(not(target_arch = "wasm32"))]
    WatchFileRequest {
        filename: PathBuf,
        //provider: Arc<Mutex<DynProvider>>,
    },

    /// A filesystem notification event was received
    #[cfg(not(target_arch = "wasm32"))]
    FsNotify(notify::Event),

    FsNotifyError(String),
}

impl Default for Event {
    fn default() -> Self {
        Event::Empty
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(CommandKind))]
pub enum Command {
    Reload,
}
