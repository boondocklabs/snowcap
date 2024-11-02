pub mod dispatch;
pub mod error;
pub mod event;
pub mod handle;
pub mod manager;
pub mod message;
pub mod registry;
pub mod test;

pub mod http;
pub mod timing;

use async_trait::async_trait;
use event::ModuleEvent;
use handle::ModuleHandle;
use iced::{
    advanced::graphics::futures::{MaybeSend, MaybeSync},
    Task,
};
use message::{ModuleMessage, ModuleMessageKind};
use tracing::{debug, debug_span, Instrument as _};

use crate::{
    parser::module::{ModuleArgument, ModuleArguments},
    NodeId, NodeRef,
};

pub(crate) type HandleId = u64;
pub(crate) type DynModule<E> = Box<dyn Module<Event = E>>;

/// Data passed to module init method
#[derive(Debug)]
pub struct ModuleAsyncInitData {
    /// Tree NodeId that this module belongs to
    node_id: NodeId,
}

/// Module initialization trait. This is used to construct new
/// modules by the ModuleManager and register them with the event
/// dispatcher.
pub trait ModuleInit: Default + Sized + Module + 'static {
    fn new(id: HandleId) -> ModuleHandle<Self::Event> {
        ModuleHandle::new(id, Self::default())
    }
}

/// Module trait, implemented by each module.
pub trait Module: ModuleAsync + MaybeSend + MaybeSync + std::fmt::Debug {
    /// Module startup.
    ///
    /// Calls synchronous initialization functions in the module implementation,
    /// and returns an async Task to the iced runtime to call the async module init() method.
    fn start(
        &mut self,
        handle: ModuleHandle<Self::Event>,
        node_id: NodeId,
        args: ModuleArguments,
    ) -> Task<ModuleMessage>
    where
        Self::Event: 'static,
    {
        let handle_id = handle.id();

        let span = debug_span!("module-init");

        // Perform synchronous module initialization
        span.in_scope(|| {
            debug!("Initializing module");
            self.init_tree(handle.subtree());
        });

        // Convert the handle into a raw handle that can cross await boundaries
        let handle = handle.into_raw();

        Task::future(async move {
            // Get a write lock on the module handle, and proxy to the
            // ModuleAsync impl async init() method of the underlying module.
            match handle.try_module_mut() {
                Ok(mut module) => {
                    let init_data = ModuleAsyncInitData { node_id };

                    span.in_scope(|| debug!("Async init"));

                    let event = module.init(args, init_data).instrument(span).await;
                    ModuleMessage::from((handle_id, event))
                }
                Err(e) => ModuleMessage::from((handle_id, crate::Error::from(e))),
            }
        })
    }

    fn init_tree(&mut self, _tree: Option<&NodeRef<Self::Event>>) {}

    fn on_event(&mut self, _event: Self::Event) -> Task<ModuleMessageKind> {
        Task::none()
    }

    fn on_message(&mut self, _message: ModuleMessageKind) -> Task<ModuleMessageKind> {
        Task::none()
    }
}

#[async_trait]
pub trait ModuleAsync {
    type Event: ModuleEvent;
    async fn init(&mut self, args: ModuleArguments, init_data: ModuleAsyncInitData) -> Self::Event;
}
