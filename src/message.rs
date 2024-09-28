use iced::widget::markdown::Url;

/// Represents a message that can be passed within the application.
/// This enum encapsulates both application-specific messages and other events.
#[derive(Debug, Clone)]
pub enum Message<AppMessage> {
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
    Button,

    /// A message variant for handling toggler events.
    Toggler(bool),

    /// A pick list was selected
    PickListSelected(String),
}
