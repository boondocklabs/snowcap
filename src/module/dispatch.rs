use std::{any::Any, sync::Arc};

use iced::Task;

use crate::{ConversionError, Error, NodeId};

use super::{event::ModuleEvent, message::ModuleMessage, ModuleHandle};

/// Module event dispatcher which provides type erasure of the concrete [`ModuleEvent`] type.
///
/// A closure is used that accepts an Arc<Box<dyn Any>> of a [`ModuleEvent`], and
/// downcasts it back to the original concrete type and passes it to the module's
/// event handler method.
///
/// The returned task from the on_event() handler of the module is mapped to
/// wrap the [`ModuleMessageKind`] to a [`ModuleMessage`] containing the HandleId,
/// and the closure returns a Task<ModuleMessage>.
pub struct ModuleDispatch {
    /// Start the module
    start: Box<dyn FnMut(NodeId) -> Task<ModuleMessage>>,

    // Closure function that takes a dyn Any of a [`ModuleEvent`] impl
    // this closure will downcast the Any
    event_dispatch: Box<dyn FnMut(Arc<Box<dyn Any + Send + Sync>>) -> Task<ModuleMessage>>,

    // Message dispatch
    message_dispatch: Box<dyn FnMut(ModuleMessage) -> Task<ModuleMessage>>,
}

impl ModuleDispatch {
    pub fn new<E: ModuleEvent + 'static>(handle: ModuleHandle<E>) -> Self {
        let event_handle = handle.clone();
        // Create an Arc wrapped closure that downcasts a dyn Any event to our concrete event type `E`
        let event_dispatch = Box::new(move |event: Arc<Box<dyn Any + Send + Sync>>| {
            let event = Arc::into_inner(event).unwrap();

            // Downcast the event back to the concrete type `E` provided to [`ModuleDispatch::new()`]
            match event.downcast::<E>() {
                Ok(event) => {
                    let mut module = event_handle.try_module_mut().unwrap();
                    let task = module.on_event(*event);
                    let handle_id = event_handle.id();
                    task.map(move |kind| ModuleMessage::new(handle_id, kind))
                }
                Err(e) => {
                    tracing::error!("Unexpected event type attempting to downcast: {e:?}");

                    // Create a task that emits a module error message
                    Task::done(ModuleMessage::from((
                        event_handle.id(),
                        Error::from(ConversionError::Downcast(
                            "unexpected ModuleEvent type".into(),
                        )),
                    )))
                }
            }
        });

        // Create a `message_dispatch` closure to forward a message to the on_message() handler of a module
        let message_handle = handle.clone();
        let handle_id = message_handle.id();
        let message_dispatch = Box::new(move |mut message: ModuleMessage| {
            let mut module = message_handle.try_module_mut().unwrap();
            let task = module.on_message(message.take_kind());
            task.map(move |kind| ModuleMessage::new(handle_id, kind))
        });

        let start_handle = handle.clone();
        let handle_id = start_handle.id();
        let start = Box::new(move |node_id| {
            let mut module = start_handle.try_module_mut().unwrap();
            let task = module.start(start_handle.clone(), node_id);
            //task.map(move |kind| ModuleMessage::new(handle_id, kind))
            task
        });

        Self {
            start,
            event_dispatch,
            message_dispatch,
        }
    }

    /// Handle a dyn Any event destined to this module, and return the Task provided by the module
    pub fn handle_event(&mut self, event: Arc<Box<dyn Any + Send + Sync>>) -> Task<ModuleMessage> {
        (self.event_dispatch)(event)
    }

    /// Handle a ModuleMessage destined to this module.
    pub fn handle_message(&mut self, message: ModuleMessage) -> Task<ModuleMessage> {
        (self.message_dispatch)(message)
    }

    /// Start the module
    pub fn start(&mut self, node_id: NodeId) -> Task<ModuleMessage> {
        (self.start)(node_id)
    }
}
