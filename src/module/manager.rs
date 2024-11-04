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

use crate::{module::argument::ModuleArguments, NodeId};

use super::{
    dispatch::ModuleDispatch,
    error::ModuleError,
    internal::ModuleInit,
    message::{ModuleMessage, ModuleMessageContainer, Topic},
    registry::ModuleRegistry,
    Module, ModuleHandleId,
};

/// Manages dynamic dispatch of messages between the [`crate::Snowcap`] engine and module instances.
/// Allows for registration of modules with the global [`ModuleRegistry`].
pub struct ModuleManager {
    /// HashMap of HandleId to a ModuleDispatch instance
    /// for dispatching event messages with type erasure
    dispatchers: HashMap<ModuleHandleId, ModuleDispatch>,

    /// Channel subscriptions. Each [`Topic`] key has a [`Vec`] of [`ModuleHandleId`]
    /// to manage a list of handles to forward each published message to.
    subscriptions: HashMap<Topic, Vec<ModuleHandleId>>,

    /// Map of [`ModuleHandleId`] to [`NodeId`], for dispatching module data to nodes
    nodes: HashMap<ModuleHandleId, NodeId>,
}

impl ModuleManager {
    pub fn new() -> Self {
        // Register internal modules
        Self::register_internal();

        Self {
            dispatchers: HashMap::new(),
            subscriptions: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    /// Register a module with the global [`ModuleRegistry`]
    pub fn register<T: ModuleInit + Module>(&self, name: &str) {
        ModuleRegistry::register::<T>(name);
    }

    /// Register internal modules with the registry
    fn register_internal() {
        ModuleRegistry::register::<super::file::FileModule>("file");
        ModuleRegistry::register::<super::http::HttpModule>("http");
        ModuleRegistry::register::<super::timing::TimingModule>("timing");
    }

    /// Create a new module instance, start it, and return a tuple of the [`ModuleHandleId`] and init [`iced::Task`]
    pub fn instantiate(
        &mut self,
        name: &String,
        args: ModuleArguments,
    ) -> Result<(ModuleHandleId, Task<ModuleMessageContainer>), ModuleError> {
        let name = name.clone();
        // Get the descriptor from the [`ModuleRegistry']
        ModuleRegistry::get(&name, move |descriptor| {
            let mut dispatch = (descriptor.new)();
            let task = dispatch.start(&args);

            let handle_id = dispatch.handle_id();

            // Register this module instance dispatcher with the manager
            self.dispatchers.insert(dispatch.handle_id(), dispatch);

            Ok((handle_id, task))
        })
    }

    /// Dispatch an event to a module specified by HandleId. The event will be downcast back to
    /// the concrete type by [`ModuleDispatch`]
    fn dispatch_message(
        &mut self,
        id: ModuleHandleId,
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

    /// Subscribe a module to a [`Topic`]
    fn subscribe(&mut self, handle_id: ModuleHandleId, channel: &Topic) {
        debug!("Module HandleId {} subscribed to {:?}", handle_id, channel);

        self.subscriptions
            .entry(channel.clone())
            .or_insert(Vec::new())
            .push(handle_id);
    }

    /// Add the [`NodeId`] associated with a [`ModuleHandleId`]
    pub fn set_module_node(&mut self, handle_id: ModuleHandleId, node_id: NodeId) {
        self.nodes.insert(handle_id, node_id);
    }

    /// Get the [`NodeId`] associated with a [`ModuleHandleId`]
    pub fn get_module_node(&mut self, handle_id: ModuleHandleId) -> Option<NodeId> {
        self.nodes.get(&handle_id).copied()
    }

    /// Handle a ModuleMessage. This is called from [`Snowcap::update()`] on receipt of a [`ModuleMessage`].
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
                let node_id = self.get_module_node(message.handle_id());
                error!(
                    "Module HandleID: {} NodeID: {node_id:?} Error: {e:#?}",
                    message.handle_id()
                );

                // TODO: Determine if the module needs to be restarted,
                // handle backoff, and try again
                Task::none()
            }

            // Module is requesting a subscription to a [`Topic`]
            ModuleMessage::Subscribe(topic) => {
                self.subscribe(message.handle_id(), topic);
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

            // Data received from a module
            ModuleMessage::Data(data) => {
                println!(
                    "Received {:?} data from Module HandleID: {}",
                    data.kind(),
                    message.handle_id()
                );

                // Find the NodeId
                let node_id = self.nodes.get(&message.handle_id());
                println!("Update data in {node_id:?}");

                Task::none()
            }

            // Messages not handled internally should be dispatched to the module specified in message.handle_id
            _ => self.dispatch_message(message.handle_id(), message),
        }
    }
}
