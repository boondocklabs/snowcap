use std::{path::PathBuf, sync::Arc};

use iced::widget::markdown::Url;
use parking_lot::Mutex;

use crate::{
    data::provider::{Provider, ProviderEvent},
    parser::ElementId,
};

/// Represents a message that can be passed within the application.
/// This enum encapsulates both application-specific messages and other events.
#[derive(Debug, Clone)]
pub enum Message<AppMessage = Event> {
    // Provide a default empty state, to allow std::mem::take()
    // to take ownership of a variant
    Empty,

    /// A variant that contains an application-specific message.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The type of the application-specific message.
    App(AppMessage),

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

    Event(Event),
}

impl<AppMessage> Default for Message<AppMessage> {
    fn default() -> Self {
        Message::Empty
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Empty,

    // Request a file be added to the watcher
    #[cfg(not(target_arch = "wasm32"))]
    WatchFileRequest {
        filename: PathBuf,
        provider: Arc<Mutex<dyn Provider>>,
    },

    Debug(String),

    /// A filesystem notification event was received
    #[cfg(not(target_arch = "wasm32"))]
    FsNotify(notify::Event),

    FsNotifyError(String),

    Provider(ProviderEvent),
}

impl Default for Event {
    fn default() -> Self {
        Event::Empty
    }
}
