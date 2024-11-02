use std::{
    any::Any,
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

use iced::Task;
use tracing::{debug, error};

use crate::parser::module::{ModuleArgument, ModuleArguments};

use super::{
    dispatch::ModuleDispatch,
    error::ModuleError,
    message::{Channel, ModuleMessage, ModuleMessageKind},
    HandleId, Module, ModuleInit,
};

pub struct ModuleManager {
    next_id: AtomicU64,

    // HashMap of HandleId to a ModuleDispatch instance
    // for dispatching event messages with type erasure
    dispatchers: HashMap<HandleId, ModuleDispatch>,

    /// Channel subscriptions
    subscriptions: HashMap<Channel, Vec<HandleId>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            dispatchers: HashMap::new(),
            subscriptions: HashMap::new(),
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
    pub fn start(&mut self, id: HandleId, args: &ModuleArguments) -> Task<ModuleMessage> {
        if let Some(dispatcher) = self.dispatchers.get_mut(&id) {
            dispatcher.start(0, args)
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

    pub fn subscribe(&mut self, channel: Channel, handle_id: HandleId) {
        debug!("Module HandleId {} subscribed to {:?}", handle_id, channel);

        self.subscriptions
            .entry(channel)
            .or_insert(Vec::new())
            .push(handle_id);
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

            ModuleMessageKind::Subscribe(channel) => {
                self.subscribe(channel, message.handle_id());
                Task::none()
            }

            ModuleMessageKind::Publish(msg) => {
                if let Some(subs) = self.subscriptions.get(&msg.channel) {
                    let mut tasks = Vec::new();
                    for sub in subs {
                        // Create a task which sends a publish message for this subscriber
                        // and push to the vec of tasks to include in the batch
                        //
                        let m = ModuleMessage::new(*sub, ModuleMessageKind::Published(msg.clone()));

                        tasks.push(Task::done(m));
                    }
                    Task::batch(tasks)
                } else {
                    Task::none()
                }
            }

            ModuleMessageKind::Published(msg) => {
                // Dispatch published messages to subscribers

                // Need to reconstruct the message, as the match uses take_kind(), leaving
                // a default None kind in its place
                let m = ModuleMessage::new(message.handle_id(), ModuleMessageKind::Published(msg));

                self.dispatch_message(message.handle_id(), m)
            }
        }
    }
}
