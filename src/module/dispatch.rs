use std::{any::Any, sync::Arc};

use iced::Task;

use crate::{ConversionError, Error};

use super::{event::ModuleEvent, message::ModuleMessage, ModuleHandle};

/// Module event dispatcher which provides type erasure of the conrete [`ModuleEvent`] type.
///
/// A closure is used that accepts an Arc<Box<dyn Any>> of a [`ModuleEvent`], and
/// downcasts it back to the original concrete type and passes it to the module's
/// event handler method.
///
/// The returned task from the on_event() handler of the module is mapped to
/// wrap the [`ModuleMessageKind`] to a [`ModuleMessage`] containing the HandleId,
/// and the closure returns a Task<ModuleMessage>.
pub struct ModuleDispatch {
    handle_dispatch: Box<dyn FnMut(Box<dyn Any + Send + Sync>)>,

    // Closure function that takes a dyn Any of a [`ModuleEvent`] impl
    // this closure will downcast the Any
    event_dispatch: Box<dyn FnMut(Arc<Box<dyn Any + Send + Sync>>) -> Task<ModuleMessage>>,
}

impl ModuleDispatch {
    pub fn new<E: ModuleEvent + 'static>(handle: ModuleHandle<E>) -> Self {
        let handle_dispatch = Box::new(move |handle: Box<dyn Any + Send + Sync>| {
            handle.downcast::<ModuleHandle<E>>();
        });

        // Create an Arc wrapped closure that downcasts a dyn Any event to our concrete event type `E`
        let event_dispatch = Box::new(move |event: Arc<Box<dyn Any + Send + Sync>>| {
            let event = Arc::into_inner(event).unwrap();

            // Downcast the event back to the concrete type `E` provided to [`ModuleDispatch::new()`]
            match event.downcast::<E>() {
                Ok(event) => {
                    let mut module = handle.try_module_mut().unwrap();
                    let task = module.on_event(*event);
                    let handle_id = handle.id();
                    task.map(move |kind| ModuleMessage::new(handle_id, kind))
                }
                Err(e) => {
                    tracing::error!("Unexpected event type attempting to downcast: {e:?}");

                    // Create a task that emits a module error message
                    Task::done(ModuleMessage::from((
                        handle.id(),
                        Error::from(ConversionError::Downcast(
                            "unexpected ModuleEvent type".into(),
                        )),
                    )))
                }
            }
        });
        Self {
            handle_dispatch,
            event_dispatch,
        }
    }

    /// Handle a dyn Any event destined to this module, and return the Task provided by the module
    pub fn handle_event(&mut self, event: Arc<Box<dyn Any + Send + Sync>>) -> Task<ModuleMessage> {
        (self.event_dispatch)(event)
    }
}
