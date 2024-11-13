//! Module framework for creating dynamic functionality that can be referenced in Snowcap grammar.
//! Snowcap includes a set of builtin modules to access network and file resources.
//! * file
//! * http
//! * timing

pub mod argument;
pub mod dispatch;
pub mod error;
pub mod event;
pub mod handle;
pub mod manager;
pub mod message;
pub mod registry;

pub mod file;
pub mod http;
pub mod timing;

pub mod data;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use data::ModuleData;
use error::ModuleError;
use event::ModuleEvent;
use handle::ModuleHandle;
use iced::{
    advanced::graphics::futures::{MaybeSend, MaybeSync},
    Task,
};
use internal::ModuleInternal;
use salish::Message;

use crate::{
    message::module::{ModuleMessageData, Topic, TopicMessage},
    module::argument::ModuleArguments,
    NodeRef,
};

/// Module instance handle ID
pub(crate) type ModuleHandleId = u64;

/// Dynamic dispatch to an implementation of the sealed [`ModuleInternal`] trait
pub(crate) type DynModule<E, D> = Box<dyn ModuleInternal<Event = E, Data = D>>;

mod internal {
    //! Sealed Module traits for initializing modules, and dispatching messages

    use crate::{message::module::ModuleMessageData, Error, Source};

    use super::{
        argument::ModuleArguments, data::ModuleData, handle::ModuleHandle, message::ModuleMessage,
        Module, ModuleHandleId, ModuleInitData,
    };
    use iced::Task;
    use salish::{message::Destination, Message};
    use tracing::{debug, debug_span, instrument, trace, Instrument as _};

    /// Module startup, and dynamic dispatch of [`ModuleMessage`] from [`crate::module::dispatch::ModuleDispatch`] instances
    /// associated with each instantiation of this [`Module`]
    pub trait ModuleInternal: Module {
        /// Module startup.
        ///
        /// Calls synchronous initialization functions in the module implementation,
        /// and returns an async Task to the iced runtime to call the async module init() method.
        fn start(
            &mut self,
            handle: ModuleHandle<'static, Self::Event, Self::Data>,
            args: ModuleArguments,
            event_addr: u64,
        ) -> Task<Message>
        where
            Self::Event: 'static,
        {
            let handle_id = handle.id();

            let span = debug_span!("module", module = handle.name());

            // Perform synchronous module initialization
            span.in_scope(|| {
                //self.init_tree(handle.subtree());
            });

            let module_name = handle.name().clone();

            Task::future(
                async move {
                    // Get a write lock on the module handle, and proxy to the
                    // ModuleAsync impl async init() method of the underlying module.
                    match handle.try_module_mut() {
                        Ok(mut module) => {
                            let init_data = ModuleInitData {};

                            debug!("Module async init {}", args);

                            let result = module
                                .init(args, init_data)
                                .instrument(debug_span!("init", module = module_name))
                                .await;

                            match result {
                                Ok(event) => {
                                    println!("RECEIVED EVENT DURING STARTUP");
                                    Message::unicast(event)
                                        .with_dest(Destination::Endpoint(event_addr))
                                        .with_source(Source::Module(handle_id))
                                }
                                Err(e) => Message::broadcast(ModuleMessage::new(
                                    handle_id,
                                    Message::unicast(Box::new(Error::from(e))),
                                ))
                                .with_source(Source::Module(handle_id)),
                            }
                        }
                        Err(e) => Message::broadcast(ModuleMessage::new(
                            handle_id,
                            Message::unicast(Box::new(crate::Error::from(e)))
                                .with_source(Source::Module(handle_id)),
                        )),
                    }
                }
                .instrument(span),
            )
        }

        /// Handle an incoming message sent to this module instance from the dispatcher
        #[instrument(name = "module")]
        fn handle_message(
            &mut self,
            module_name: &String,
            message: ModuleMessageData,
        ) -> Task<ModuleMessageData>
        where
            Self::Event: 'static,
        {
            trace!("{:?}", message);
            /*
            match message {
                ModuleMessageData::Event(event) => {
                    let event = Arc::into_inner(event).unwrap();

                    // Downcast the event back to the concrete type specified by the
                    // associated type [`Module::Event`] from the module implementation.

                    match event.downcast::<Self::Event>() {
                        Ok(event) => {
                            debug!("on_event {:?}", event);
                            self.on_event(*event)
                        }
                        Err(e) => {
                            tracing::error!("Unexpected event type attempting to downcast: {e:?}");

                            // Create a task that emits a module error message
                            Task::done(ModuleMessageData::from(Error::from(
                                ConversionError::Downcast("unexpected ModuleEvent type".into()),
                            )))
                        }
                    }
                }
                ModuleMessageData::Published(publish_message) => {
                    debug!("on_subscription {}", publish_message);
                    self.on_subscription(
                        publish_message.topic.clone(),
                        publish_message.message.clone(),
                    )
                }
                _ => {
                    debug!("on_message {:?}", message);
                    self.on_message(message)
                }
            }
            */

            Task::none()
        }

        /// Get a Task to send data from this module to the Snowcap engine
        fn send_data(&self, data: Self::Data) -> Task<Message> {
            let data: Box<dyn ModuleData> = Box::new(data);
            Task::done(Message::unicast(data))
        }

        fn event(&self, event: Self::Event) -> Task<Self::Event>
        where
            Self::Event: 'static,
        {
            Task::done(event)
        }
    }

    /// Module instantiation and registration. This is used to construct new
    /// modules by the [`crate::module::manager::ModuleManager`] and registering them with the event
    /// dispatcher.
    pub trait ModuleInit: Default + Sized + Module + 'static {
        /// Instantiate this module
        fn new(name: String, id: ModuleHandleId) -> ModuleHandle<'static, Self::Event, Self::Data> {
            ModuleHandle::new(name, id, Self::default())
        }

        /// Get the fully qualified type name of the [`Module`] implementation
        fn type_name() -> &'static str {
            std::any::type_name::<Self>()
        }

        /// Get the fully qualified type name of the associated [`ModuleEvent`] type of this [`Module`]
        fn event_name() -> &'static str {
            std::any::type_name::<Self::Event>()
        }
    }
}

/// Implement [`ModuleInit`] on anything implementing [`Module`]
impl<T> internal::ModuleInit for T where T: Module + Default + 'static {}

/// Implement [`ModuleInternal`] on anything implementing [`Module`]
impl<T> internal::ModuleInternal for T where T: Module {}

/// Data passed to module init method
#[derive(Debug)]
pub struct ModuleInitData {}

/// Module trait, implemented by each module.
#[async_trait]
pub trait Module: MaybeSend + MaybeSync + std::fmt::Debug {
    /// Internal module event type, for intra-module messaging
    type Event: ModuleEvent;

    type Data: ModuleData + 'static;

    /// Async Module initialization method which is implemented by each available module.
    /// The set of arguments parsed from the grammar is included in `args` as [`crate::module::argument::ModuleArguments`]
    async fn init(
        &mut self,
        args: ModuleArguments,
        init_data: ModuleInitData,
    ) -> Result<Self::Event, ModuleError>;

    fn init_tree(&mut self, _tree: Option<&NodeRef>) {}

    /// Called when a [`ModuleEvent`] is received for this module.
    /// This is used for intra-module communication, using the associated
    /// type [`Module::Event`] defined by the module implementation.
    ///
    /// An [`iced::Task`] must be returned in response to the message,
    /// which may emit [`salish::Message`] messages, which will be dispatched
    /// by the [`crate::Snowcap`] engine.
    ///
    /// If no work needs to be done in response
    /// to the message, return [`iced::Task::none()`]
    fn on_event(&mut self, _event: Self::Event) -> Task<Message> {
        Task::none()
    }

    /// Called when a subscription message is received on a [`Topic`] that this [`Module`] has subscribed to.
    /// Subscriptions are created by issuing a [`ModuleMessage::Subscribe`] from an [`iced::Task`] with the [`Topic`] of interest,
    /// and it will be registered into the [`crate::module::manager::ModuleManager`].
    ///
    /// An [`iced::Task`] must be returned in response to the message,
    /// which may emit [`ModuleMessage`] messages, which will be dispatched
    /// by the [`crate::Snowcap`] engine.
    ///
    /// If no work needs to be done in response
    /// to the message, return [`iced::Task::none()`]
    fn on_subscription(&mut self, _topic: Topic, _message: TopicMessage) -> Task<Message> {
        Task::none()
    }

    /// Called when a [`ModuleMessage`] is received for this [`Module`].
    /// Note that [`ModuleMessage::Published`] and [`ModuleMessage::Event`] types are not forwarded
    /// to this method, as they are forwarded to [`Module::on_subscription()`] and [`Module::on_event()`] respectively.
    ///
    /// An [`iced::Task`] must be returned in response to the message,
    /// which may emit [`ModuleMessage`] messages, which will be dispatched
    /// by the [`crate::Snowcap`] engine.
    ///
    /// If no work needs to be done in response
    /// to the message, return [`iced::Task::none()`]
    fn on_message(&mut self, _message: ModuleMessageData) -> Task<ModuleMessageData> {
        Task::none()
    }
}
