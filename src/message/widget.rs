//! Widget Messages

use crate::{parser::ElementId, NodeId};
use iced::widget::scrollable::Viewport;
use url::Url;

#[derive(Clone, Debug)]
pub struct WidgetMessage {
    pub node_id: NodeId,
    pub element_id: Option<ElementId>,
    pub event: WidgetEvent,
}

impl WidgetMessage {
    pub fn new(node_id: NodeId, element_id: Option<ElementId>, event: WidgetEvent) -> Self {
        Self {
            node_id,
            element_id,
            event,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WidgetEvent {
    /// Markdown widget clicked URL
    Markdown(Url),

    /// Button Pressed
    ButtonPress,

    /// Toggler toggled
    Toggler(bool),

    /// A pick list was selected
    PickListSelected(String),

    /// Slider value changed
    SliderChanged(i32),
    SliderReleased(i32),
    Scrolled(Viewport),
}

/*
impl From<WidgetMessage> for Message {
    fn from(widget_message: WidgetMessage) -> Self {
        Message {
            dest: MessageEndpoint::Internal,
            data: MessageData::Widget(widget_message),
        }
    }
}

impl From<Message> for WidgetMessage {
    fn from(message: Message) -> Self {
        if let MessageData::Widget(widget_message) = message.data {
            widget_message
        } else {
            panic!()
        }
    }
}

impl<'a> From<&'a mut Message> for &'a WidgetMessage {
    fn from(message: &'a mut Message) -> Self {
        if let MessageData::Widget(widget_message) = &**message {
            widget_message
        } else {
            panic!()
        }
    }
}
*/

/*
impl<'a> AsRef<WidgetMessage> for &'a mut Message {
    fn as_ref(&self) -> &WidgetMessage {
        if let MessageData::Widget(widget_message) = &self.data {
            widget_message
        } else {
            panic!()
        }
    }
}
*/
