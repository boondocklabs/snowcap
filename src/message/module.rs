use std::sync::Arc;

use colored::Colorize as _;

use crate::module::data::ModuleData;

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

#[derive(Default, Clone, Debug)]
pub enum ModuleMessageData {
    // Provide a default so mem::take() can be used to take ownership
    // of a variant, leaving None in its place
    #[default]
    None,
    Debug(&'static str),
    Error(Arc<Box<dyn std::error::Error + Send + Sync>>),
    //Event(Box<dyn Any + Send + Sync>),
    /// Module requesting a subscription to a channel
    Subscribe(Topic),

    /// Publish a message to a channel
    Publish(PublishMessage),

    /// A published message being sent to a module
    Published(PublishMessage),

    /// Data updated by module
    Data(Arc<Box<dyn ModuleData>>),
}
