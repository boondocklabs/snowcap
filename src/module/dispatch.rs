use std::any::Any;

use iced::Task;
use salish::{filter::SourceFilter, EndpointAddress as _, Message};

use crate::{module::argument::ModuleArguments, Source};

use super::{data::ModuleData, event::ModuleEvent, ModuleHandle, ModuleHandleId};

/// Module event dispatcher which provides type erasure of the concrete [`ModuleEvent`] type.
///
/// A closure is used that accepts an Arc<Box<dyn Any>> of a [`ModuleEvent`], and
/// downcasts it back to the original concrete type and passes it to the module's
/// event handler method.
pub struct ModuleDispatch {
    handle_id: ModuleHandleId,

    /// Start the module
    start: Box<dyn for<'b> FnMut(&'b ModuleArguments) -> Task<Message> + Send + Sync>,

    /// Vec which holds endpoints created for this module to keep them alive. Once this Vec
    /// is dropped, all of the endpoints will be deregistered from the [`MessageRouter`]
    _endpoints: Vec<Box<dyn Any + Send>>,
}

impl Drop for ModuleDispatch {
    fn drop(&mut self) {
        println!("DISPATCHER DROPPED {}", self.handle_id);
    }
}

impl std::fmt::Debug for ModuleDispatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleDispatch").finish()
    }
}

impl ModuleDispatch {
    /// Create a new [`ModuleDispatch`] instance for a [`ModuleHandle`]. Creates closures
    /// to provide type erasure of the [`ModuleEvent`] for instantiation and message dispatch.
    /// This allows the dispatcher to be stored in collections of arbitrary module types using `dyn`.
    pub fn new<E: ModuleEvent + 'static, D: ModuleData + 'static>(
        handle: ModuleHandle<'static, E, D>,
    ) -> Self {
        let start_handle = handle.clone();
        let handle_id = handle.id();

        let router = handle.router().unwrap();

        // Create an event endpoint that calls [`Module::on_event()`] for each event received by the endpoint
        // This endpoint is registered to receive `ModuleEvent` associated type messages from the specific [`Module`] implementation
        let event_endpoint = router
            .create_endpoint::<E>()
            .filter(SourceFilter::default().add(Source::Module(handle_id)))
            .message(move |_source, event| {
                let mut module = handle.try_module_mut().unwrap();
                module
                    .on_event(event)
                    .map(move |m| m.with_source(Source::Module(handle_id)))
            });

        // Get the event endpoint address to pass into [`ModuleInit::start()`].
        // This address routes events back into the [`Module::on_event()`] method
        let event_addr = event_endpoint.addr();

        // Keep the endpoints alive in a vec of boxed dyn Any
        let endpoints: Vec<Box<dyn Any + Send>> = vec![Box::new(event_endpoint)];

        // Create a `start` closure to proxy to [`ModuleInternal::start()`]
        let start = Box::new(move |args: &ModuleArguments| {
            let mut module = start_handle.try_module_mut().unwrap();
            let task = module.start(start_handle.clone(), args.clone(), event_addr);

            // Return the init Task of this module
            task
        });

        Self {
            handle_id,
            start,
            _endpoints: endpoints,
        }
    }

    /// Get the [`HandleId`] associated with the [`ModuleHandle`] for this dispatcher
    pub fn handle_id(&self) -> ModuleHandleId {
        self.handle_id
    }

    /// Starts the module, calling [`crate::module::internal::ModuleInternal::start()`]
    /// which returns an [`iced::Task`] which calls into the async fn [`super::Module::init()`]
    /// implemented by the module.
    pub fn start(&mut self, args: &ModuleArguments) -> Task<Message> {
        (self.start)(args)
    }
}
