//! Module instance lifecycle management
//!
//! Manages dynamic dispatch of messages between the [`crate::Snowcap`] engine and module instances.
//! Allows for registration of modules with the global [`ModuleRegistry`].
//!
//! Custom modules can be registered using [`ModuleManager::register()`]. For example if you have a struct named `MyModule` which implements [`Module`],
//! it can be registered into the [`ModuleManager`] of a [`crate::Snowcap`] instance, specifying the generic to the register method:
//!
//! ```ignore
//! snowcap.modules().register::<MyModule>("custom-module");
//! ```

use std::{any::Any, collections::HashMap, sync::Arc};

use arbutus::{TreeNode as _, TreeNodeRef as _};
use iced::Task;
use salish::{endpoint::Endpoint, filter::SourceFilter, router::MessageRouter, Message};
use tracing::{debug, error, warn};

use crate::{
    message::module::Topic,
    module::{argument::ModuleArguments, data::ModuleData},
    NodeId, NodeRef, Source,
};

use super::{
    dispatch::ModuleDispatch, error::ModuleError, internal::ModuleInit, registry::ModuleRegistry,
    Module, ModuleHandleId,
};

/// Manages dynamic dispatch of messages between the [`crate::Snowcap`] engine and module instances.
/// Allows for registration of modules with the global [`ModuleRegistry`].
pub struct ModuleManager {
    /// HashMap of [`ModuleHandleId`] to a [`ModuleDispatch`] instance
    /// for dispatching event messages with type erasure
    dispatchers: HashMap<ModuleHandleId, ModuleDispatch>,

    /// Channel subscriptions. Each [`Topic`] key has a [`Vec`] of [`ModuleHandleId`]
    /// to manage a list of handles to forward each published message to.
    ///
    /// TODO: Move pubsub to the [`MessageRouter`]
    subscriptions: HashMap<Topic, Vec<ModuleHandleId>>,

    /// Map of [`ModuleHandleId`] to [`NodeId`], for dispatching module data to nodes
    nodes: HashMap<ModuleHandleId, NodeId>,

    /// The [`salish::MessageRouter`] for acquiring new [`Endpoint`] instances
    router: MessageRouter<'static, Task<salish::message::Message>, Source>,

    /// Salish message endpoint to receive ModuleMessage for each instantiated module
    data_endpoints: HashMap<
        ModuleHandleId,
        Endpoint<'static, Box<dyn ModuleData>, Task<crate::Message>, Source>,
    >,

    _ep: Vec<Box<dyn Any>>,
}

impl std::fmt::Debug for ModuleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleManager").finish()
    }
}

impl ModuleManager {
    pub fn new(router: MessageRouter<'static, Task<salish::message::Message>, Source>) -> Self {
        // Register internal modules
        Self::register_internal();

        let mut manager = Self {
            dispatchers: HashMap::new(),
            subscriptions: HashMap::new(),
            nodes: HashMap::new(),
            data_endpoints: HashMap::new(),
            router,
            _ep: Vec::new(),
        };

        manager.init();

        manager
    }

    /// Initialize the [`ModuleManager`], creating data handling endpoint to update tree nodes
    fn init(&mut self) {
        /*
        let data_endpoint = self
            .router
            .create_endpoint::<Box<dyn ModuleData>>()
            .message(|source, data| {
                println!("RECEIVED DATA {data:?} FROM {source:?}");
                Task::none()
            });

        self._ep.push(Box::new(data_endpoint));
        */
    }

    /// Register a module with the global [`ModuleRegistry`]
    pub fn register<T: ModuleInit + Module>(&self, name: &str) {
        ModuleRegistry::register::<T>(name);
    }

    /// Register all internal modules with the registry
    fn register_internal() {
        ModuleRegistry::register::<super::file::FileModule>("file");
        ModuleRegistry::register::<super::http::HttpModule>("http");
        ModuleRegistry::register::<super::timing::TimingModule>("timing");
        ModuleRegistry::register::<super::sub::SubModule>("sub");

        println!("{}", ModuleRegistry);
    }

    /// Create a new module instance, start it, and return a tuple of the [`ModuleHandleId`] and init [`iced::Task`]
    pub fn instantiate(
        &mut self,
        name: &String,
        args: ModuleArguments,
    ) -> Result<(ModuleHandleId, Task<Message>), ModuleError> {
        let name = name.clone();

        // Clone the router to move into the closure
        let router = self.router.clone();

        // Get the descriptor from the [`ModuleRegistry']
        ModuleRegistry::get(&name, move |descriptor| {
            // Create a new instance of the module and get a type erased [`ModuleDispatch`] handle
            // to proxy into internal module methods.
            let mut dispatch = (descriptor.new)(router);

            // Get the init Task of the module, which calls back to the async [`Module::init()`] method
            // of the [`Module`] implementation for the requested module name.
            let task = dispatch.start(&args);

            // Get the handle ID
            let handle_id = dispatch.handle_id();

            /*
            // Get an endpoint from the router for this module, and move the [`ModuleDispatch`] into
            // the message handler closure to forward messages into the module
            let module_endpoint =
                self.router
                    .create_endpoint::<ModuleMessage>()
                    .message(move |message| {
                        println!("Module Endpoint Received {message:?}");

                        dispatch
                            .handle_message(message)
                            .map(|msg| salish::message::Message::broadcast(msg))
                    });

            self.endpoints.insert(handle_id, module_endpoint);
            */

            // Register this module instance dispatcher with the manager
            self.dispatchers.insert(dispatch.handle_id(), dispatch);

            Ok((handle_id, task))
        })
    }

    /// Subscribe a module to a [`Topic`]
    fn subscribe(&mut self, handle_id: ModuleHandleId, channel: &Topic) {
        debug!("Module HandleId {} subscribed to {:?}", handle_id, channel);

        self.subscriptions
            .entry(channel.clone())
            .or_insert(Vec::new())
            .push(handle_id);
    }

    pub fn connect_node(&mut self, handle_id: ModuleHandleId, mut noderef: NodeRef) {
        // Create a data endpoint for this module which updates tree node data
        let data_endpoint = self
            .router
            .create_endpoint::<Box<dyn ModuleData>>()
            .filter(SourceFilter::default().add(Source::Module(handle_id)))
            .message(move |source, message| {
                noderef.node_mut().data_mut().set_module_data(message);
                Task::none()
            });

        self.data_endpoints.insert(handle_id, data_endpoint);
    }

    /// Get the [`NodeId`] associated with a [`ModuleHandleId`]
    pub fn get_module_node(&mut self, handle_id: ModuleHandleId) -> Option<NodeId> {
        self.nodes.get(&handle_id).copied()
    }

    /*
    /// Handle a ModuleMessage. This is called from [`Snowcap::update()`] on receipt of a [`ModuleMessage`].
    /// Dispatch the message to the module handle using the encapsulated HandleId.
    pub fn handle_message(&mut self, message: &ModuleMessage) -> Task<ModuleMessage> {
        match message.data() {
            ModuleMessageData::None => {
                tracing::warn!("{message:?}");
                Task::none()
            }
            ModuleMessageData::Debug(msg) => {
                debug!("Module Debug Message: {}", msg);
                Task::none()
            }

            ModuleMessageData::Error(e) => {
                error!("Error: {e:#?}",);

                // TODO: Determine if the module needs to be restarted,
                // handle backoff, and try again
                Task::none()
            }

            // Module is requesting a subscription to a [`Topic`]
            ModuleMessageData::Subscribe(topic) => {
                self.subscribe(message.handle_id(), &topic);
                Task::none()
            }

            // Received a Publish message from a module. Dispatch to all modules subscribed to this topic
            ModuleMessageData::Publish(msg) => {
                // Get the subscribers to this topic
                if let Some(subs) = self.subscriptions.get(&msg.topic) {
                    let mut tasks = Vec::new();

                    // Iterate through HandleIds subscribed to this topic
                    for sub in subs {
                        // Create a task which sends a publish message to this subscriber
                        let m = ModuleMessage::new(*sub, ModuleMessageData::Published(msg.clone()));

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
            ModuleMessageData::Data(data) => {
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
            _ => self.dispatch_message(message.handle_id(), message.clone()),
        }
    }
    */
}
