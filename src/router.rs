use iced::Task;
use std::{any::TypeId, collections::HashMap};
use tracing::{debug, instrument, warn};

use crate::{
    message::handler::{HandlerWrapper, MessageHandler},
    module::ModuleHandleId,
    Message,
};

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MessageEndpoint {
    App,
    #[default]
    Internal,
    Module(ModuleHandleId),
}

/// Message Router
pub struct MessageRouter<'a> {
    handlers: HashMap<TypeId, Vec<Box<dyn for<'b> FnMut(&'b mut Message) -> Task<Message> + 'a>>>,
}

impl<'a> std::fmt::Debug for MessageRouter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get a Vec of endpoints and handler counts for debug
        let handlers_count: Vec<_> = self.handlers.iter().map(|(k, v)| (k, v.len())).collect();

        f.debug_struct("MessageRouter")
            .field("handlers", &handlers_count)
            .finish()
    }
}

impl<'a> MessageRouter<'a> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Call a [`Vec`] of handlers with a [`Message`]
    fn call_handlers(
        message: &Message,
        handlers: &mut Vec<Box<dyn FnMut(&Message) -> Task<Message>>>,
    ) -> Task<Message> {
        if handlers.len() > 1 {
            let tasks: Vec<Task<_>> = handlers
                .into_iter()
                .map(|handler| (handler)(message))
                .collect();

            Task::batch(tasks)
        } else {
            (handlers.last_mut().unwrap())(message)
        }
    }

    /// Handle a message received from the [`Snowcap::update()`] phase
    #[instrument(name = "router")]
    pub fn handle_message<'b>(&'b mut self, message: &mut Message) -> Task<Message> {
        if let Some(handlers) = self.handlers.get_mut(&message.data_type_id()) {
            if handlers.len() > 1 {
                let mut tasks = Vec::new();
                for handler in handlers {
                    let task = (handler)(message);
                    tasks.push(task)
                }
                Task::batch(tasks)
            } else {
                // Only one handler
                (handlers.last_mut().unwrap())(message)
            }
        } else {
            warn!("No Handler");
            Task::none()
        }
    }

    #[instrument(name = "router")]
    pub fn add_handler<H, W>(&mut self, handler: W)
    where
        W: HandlerWrapper<H> + Clone + std::fmt::Debug + 'a,
        H: MessageHandler + 'a,
        //for<'b> &'b mut Message: AsRef<H::Message>,
        for<'b> &'b mut Message: Into<&'b H::Message>,
        Message: From<H::Message>,
    {
        // Get the type of the handlers associated type Message
        let type_id = TypeId::of::<H::Message>();

        debug!("Handler TypeId: {type_id:?}");

        // Register a closure for dispatching messages to the handler
        let dispatch = move |msg: &mut Message| {
            if let Some(mut inner) = handler.inner() {
                let inner_message: &H::Message = msg.into();
                inner
                    .on_message(&inner_message)
                    .map(|output| Message::from(output))
            } else {
                Task::none()
            }
        };

        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(dispatch));

        debug!("Added Handler");
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::Message;

    use super::MessageRouter;

    #[traced_test]
    #[test]
    fn create() {
        let mut router = MessageRouter::new();
        let mut msg = Message::new("hello");
        let _ = router.handle_message(&mut msg);
    }
}
