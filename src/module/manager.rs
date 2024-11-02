use std::{
    any::Any,
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

use iced::Task;
use tracing::{debug, error};

use super::{
    dispatch::ModuleDispatch,
    error::ModuleError,
    message::{ModuleMessage, ModuleMessageKind},
    registry::ModuleKind,
    HandleId, Module, ModuleHandle, ModuleInit,
};

pub struct ModuleManager {
    next_id: AtomicU64,

    // HashMap of HandleId to a ModuleDispatch instance
    // for dispatching event messages with type erasure
    dispatchers: HashMap<HandleId, ModuleDispatch>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            dispatchers: HashMap::new(),
        }
    }

    /// Get a batch of init tasks for all registered modules
    pub fn get_init_tasks(&self) -> Task<ModuleMessage> {
        let tasks: Vec<Task<ModuleMessage>> = Vec::new();

        for (id, dispatcher) in &self.dispatchers {}

        Task::batch(tasks)
    }

    /// Create a new [`Module`] instance and register it with this module manager.
    ///
    /// A ModuleDispatch instance is created providing type erasure for the ModuleEvent type
    pub fn create_inner<T: ModuleInit + Module>(&mut self) -> HandleId {
        //ModuleHandle<T::Event> {
        // Get a new unique ID for this module
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Create the module, and get a ModuleHandle
        let handle = T::new(id);

        // Create a dispatcher for this module, which downcasts dyn Any events back
        // to the concrete type.
        let dispatcher = ModuleDispatch::new(handle);

        // Register the ModuleDispatch in a HashMap, so we can dispatch messages back
        // to this module using its unique HandleId
        self.dispatchers.insert(id, dispatcher);

        // Return the HandleId of this module instance
        id
    }

    /// Start the specified module
    pub fn start(&mut self, id: HandleId) -> Task<ModuleMessage> {
        if let Some(dispatcher) = self.dispatchers.get_mut(&id) {
            dispatcher.start(0)
        } else {
            // Create a task that emits a ModuleError message in the iced runtime

            Task::done(ModuleMessage::from((
                id,
                crate::Error::from(ModuleError::HandleNotFound {
                    handle_id: id,
                    msg: "event dispatch",
                }),
            )))
        }
    }

    /// Dispatch an event to a module specified by HandleId. The event will be downcast back to
    /// the concrete type by [`ModuleDispatch`]
    pub fn dispatch_event(
        &mut self,
        id: HandleId,
        event: Arc<Box<dyn Any + Send + Sync>>,
    ) -> Task<ModuleMessage> {
        if let Some(dispatcher) = self.dispatchers.get_mut(&id) {
            dispatcher.handle_event(event)
        } else {
            Task::done(ModuleMessage::from((
                id,
                crate::Error::from(ModuleError::HandleNotFound {
                    handle_id: id,
                    msg: "event dispatch",
                }),
            )))
        }
    }

    /// Dispatch an event to a module specified by HandleId. The event will be downcast back to
    /// the concrete type by [`ModuleDispatch`]
    pub fn dispatch_message(
        &mut self,
        id: HandleId,
        message: ModuleMessage,
    ) -> Task<ModuleMessage> {
        if let Some(dispatcher) = self.dispatchers.get_mut(&id) {
            dispatcher.handle_message(message)
        } else {
            Task::done(ModuleMessage::from((
                id,
                crate::Error::from(ModuleError::HandleNotFound {
                    handle_id: id,
                    msg: "event dispatch",
                }),
            )))
        }
    }

    /// Handle a ModuleMessage. This is called from the iced update phase on receipt of a ModuleMessage.
    /// Dispatch the message to the module handle using the encapsulated HandleId.
    pub fn handle_message(&mut self, mut message: ModuleMessage) -> Task<ModuleMessage> {
        match message.take_kind() {
            ModuleMessageKind::None => {
                tracing::warn!("{message:?}");
                Task::none()
            }
            ModuleMessageKind::Debug(msg) => {
                debug!("Module Debug Message: {}", msg);
                Task::none()
            }

            ModuleMessageKind::Error(e) => {
                error!("{e:#?}");
                Task::none()
            }
            // Dispatch an event message to the module
            ModuleMessageKind::Event(event) => self.dispatch_event(message.handle_id(), event),

            _ => self.dispatch_message(message.handle_id(), message),
        }
    }
}
