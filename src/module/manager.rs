//! Manages dynamic dispatch of messages between the [`crate::Snowcap`] engine and module instances.
//! Allows for registration of modules with the global [`ModuleRegistry`].
//!
//! Custom modules can be registered using [`ModuleManager::register()`]. For example if you have a struct named `MyModule` which implements [`Module`],
//! it can be registered into the [`ModuleManager`] of a [`crate::Snowcap`] instance, specifying the generic to the register method:
//!
//! ```ignore
//! snowcap.modules().register::<MyModule>("custom-module");
//! ```

use std::collections::HashMap;

use iced::Task;
use tracing::{debug, error, warn};

use crate::module::argument::ModuleArguments;

use super::{
    dispatch::ModuleDispatch,
    error::ModuleError,
    internal::ModuleInit,
    message::{ModuleMessage, ModuleMessageContainer, Topic},
    registry::ModuleRegistry,
    HandleId, Module,
};

/// Manages dynamic dispatch of messages between the [`crate::Snowcap`] engine and module instances.
/// Allows for registration of modules with the global [`ModuleRegistry`].
pub struct ModuleManager {
    /// HashMap of HandleId to a ModuleDispatch instance
    /// for dispatching event messages with type erasure
    dispatchers: HashMap<HandleId, ModuleDispatch>,

    /// Channel subscriptions
    subscriptions: HashMap<Topic, Vec<HandleId>>,
}

impl ModuleManager {
    pub fn new() -> Self {
        // Register internal modules
        Self::register_internal();

        Self {
            dispatchers: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    pub fn register<T: ModuleInit + Module>(&self, name: &str) {
        ModuleRegistry::register::<T>(name);
    }

    /// Register internal modules with the registry
    fn register_internal() {
        ModuleRegistry::register::<super::timing::TimingModule>("timing");
        ModuleRegistry::register::<super::http::HttpModule>("http");
    }

    /// Create a new module instance, start it, and return the init task
    pub fn instantiate(
        &mut self,
        name: &String,
        args: ModuleArguments,
    ) -> Result<Task<ModuleMessageContainer>, ModuleError> {
        let name = name.clone();
        // Get the descriptor from the [`ModuleRegistry']
        ModuleRegistry::get(&name, move |descriptor| {
            let mut dispatch = (descriptor.new)();
            let task = dispatch.start(&args);

            // Register this module instance dispatcher with the manager
            self.dispatchers.insert(dispatch.handle_id(), dispatch);

            Ok(task)
        })
    }

    /// Dispatch an event to a module specified by HandleId. The event will be downcast back to
    /// the concrete type by [`ModuleDispatch`]
    pub fn dispatch_message(
        &mut self,
        id: HandleId,
        message: ModuleMessageContainer,
    ) -> Task<ModuleMessageContainer> {
        if let Some(dispatcher) = self.dispatchers.get_mut(&id) {
            dispatcher.handle_message(message)
        } else {
            Task::done(ModuleMessageContainer::from((
                id,
                crate::Error::from(ModuleError::HandleNotFound {
                    handle_id: id,
                    msg: format!("{} {} dispatch message", file!(), line!()).into(),
                }),
            )))
        }
    }

    pub fn subscribe(&mut self, handle_id: HandleId, channel: &Topic) {
        debug!("Module HandleId {} subscribed to {:?}", handle_id, channel);

        self.subscriptions
            .entry(channel.clone())
            .or_insert(Vec::new())
            .push(handle_id);
    }

    /// Handle a ModuleMessage. This is called from the iced update phase on receipt of a ModuleMessage.
    /// Dispatch the message to the module handle using the encapsulated HandleId.
    pub fn handle_message(
        &mut self,
        message: ModuleMessageContainer,
    ) -> Task<ModuleMessageContainer> {
        match message.message() {
            ModuleMessage::None => {
                tracing::warn!("{message:?}");
                Task::none()
            }
            ModuleMessage::Debug(msg) => {
                debug!("Module Debug Message: {}", msg);
                Task::none()
            }

            ModuleMessage::Error(e) => {
                error!("{e:#?}");
                Task::none()
            }

            // Dispatch an event message to the module
            ModuleMessage::Subscribe(channel) => {
                self.subscribe(message.handle_id(), channel);
                Task::none()
            }

            // Received a Publish message from a module. Dispatch to all modules subscribed to this topic
            ModuleMessage::Publish(msg) => {
                // Get the subscribers to this topic
                if let Some(subs) = self.subscriptions.get(&msg.topic) {
                    let mut tasks = Vec::new();

                    // Iterate through HandleIds subscribed to this topic
                    for sub in subs {
                        // Create a task which sends a publish message to this subscriber
                        let m = ModuleMessageContainer::new(
                            *sub,
                            ModuleMessage::Published(msg.clone()),
                        );

                        // Push the task to the batch of tasks to return
                        tasks.push(Task::done(m));
                    }
                    Task::batch(tasks)
                } else {
                    warn!("Received Publish message {msg:?} with no subscribers");
                    Task::none()
                }
            }

            // Messages not handled internally should be dispatched to the module specified in message.handle_id
            _ => self.dispatch_message(message.handle_id(), message),
        }
    }
}
