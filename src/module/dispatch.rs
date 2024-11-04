use iced::Task;

use crate::module::argument::ModuleArguments;

use super::{
    data::ModuleData, event::ModuleEvent, message::ModuleMessageContainer, ModuleHandle,
    ModuleHandleId,
};

/// Module event dispatcher which provides type erasure of the concrete [`ModuleEvent`] type.
///
/// A closure is used that accepts an Arc<Box<dyn Any>> of a [`ModuleEvent`], and
/// downcasts it back to the original concrete type and passes it to the module's
/// event handler method.
pub struct ModuleDispatch {
    handle_id: ModuleHandleId,

    /// Start the module
    start: Box<dyn for<'b> FnMut(&'b ModuleArguments) -> Task<ModuleMessageContainer>>,

    // Closure function that takes a dyn Any of a [`ModuleEvent`] impl
    // this closure will downcast the Any
    //event_dispatch: Box<dyn FnMut(Arc<Box<dyn Any + Send + Sync>>) -> Task<ModuleMessage>>,
    /// Dispatch a message to this module
    message_dispatch: Box<dyn FnMut(ModuleMessageContainer) -> Task<ModuleMessageContainer>>,
}

impl ModuleDispatch {
    /// Create a new [`ModuleDispatch`] instance for a [`ModuleHandle`]. Creates closures
    /// to provide type erasure of the [`ModuleEvent`] for instantiation and message dispatch.
    /// This allows the dispatcher to be stored in collections of arbitrary module types using `dyn`.
    pub fn new<E: ModuleEvent + 'static, D: ModuleData + 'static>(
        handle: ModuleHandle<E, D>,
    ) -> Self {
        // Create a `message_dispatch` closure to forward a message to the on_message() handler of a module
        let message_handle = handle.clone();
        let handle_id = message_handle.id();
        let message_dispatch = Box::new(move |mut message: ModuleMessageContainer| {
            let mut module = message_handle.try_module_mut().unwrap();
            let task = module.handle_message(message_handle.name(), message.take_message());
            task.map(move |kind| ModuleMessageContainer::new(handle_id, kind))
        });

        let start_handle = handle.clone();
        let start: Box<dyn for<'b> FnMut(&'b ModuleArguments) -> Task<ModuleMessageContainer>> =
            Box::new(move |args| {
                let mut module = start_handle.try_module_mut().unwrap();
                let task = module.start(start_handle.clone(), args.clone());

                // Return the init Task of this module
                task
            });

        Self {
            handle_id: handle.id(),
            start,
            message_dispatch,
        }
    }

    /// Get the [`HandleId`] associated with the [`ModuleHandle`] for this dispatcher
    pub fn handle_id(&self) -> ModuleHandleId {
        self.handle_id
    }

    /// Starts the module, calling [`crate::module::internal::ModuleInternal::start()`]
    /// which returns an [`iced::Task`] which calls into the async fn [`super::Module::init()`]
    /// implemented by the module.
    pub fn start(&mut self, args: &ModuleArguments) -> Task<ModuleMessageContainer> {
        (self.start)(args)
    }

    /// Handle a ModuleMessage destined to this module.
    pub fn handle_message(
        &mut self,
        message: ModuleMessageContainer,
    ) -> Task<ModuleMessageContainer> {
        (self.message_dispatch)(message)
    }
}
