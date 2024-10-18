use std::any::Any;

use crate::{message::EventDiscriminants, ConversionError, Error};
use iced::Task;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod fsnotify;
pub(crate) mod provider;

pub trait EventHandler<M>
where
    M: std::fmt::Debug,
{
    type Event: Any + Sized;
    type State: Any + Sized;

    fn handle(&self, event: Self::Event, state: Self::State) -> Result<Task<M>, Error>;
}

pub struct DynamicHandlerAdapter<'a, M, E, S>
where
    E: Any,
    S: Any,
    M: std::fmt::Debug,
{
    pub handler: Box<dyn EventHandler<M, Event = E, State = S> + 'a>, // The original event handler with the concrete type E
}

impl<'a, M, E, S> DynamicHandlerAdapter<'a, M, E, S>
where
    E: Any + 'a,
    S: Any + 'a,
    M: std::fmt::Debug,
{
    pub fn new(handler: impl EventHandler<M, Event = E, State = S> + 'a) -> Self {
        // Box the original handler with a concrete type to erase the associated type
        let handler = Box::new(handler);

        // The inner DynamicHandler can be used as a <dyn EventHandler<Type = Box<dyn Any>>>
        //let inner = DynamicHandler::new(handler);

        Self { handler }
    }
}

impl<'a, M, E, S> EventHandler<M> for DynamicHandlerAdapter<'a, M, E, S>
where
    E: Any + 'a,
    S: Any + 'a,
    M: std::fmt::Debug,
{
    type Event = Box<dyn Any>; // The adapted event type is now Box<dyn Any>
    type State = Box<dyn Any>;

    fn handle(
        &self,
        event: Self::Event,
        state: Self::State,
    ) -> Result<iced::Task<M>, crate::Error> {
        let event = event.downcast::<E>().map_err(|_| {
            Error::Conversion(ConversionError::Downcast(
                "DynamicHandlerAdapter Event".into(),
            ))
        })?;

        let state = state.downcast::<S>().map_err(|_| {
            Error::Conversion(ConversionError::Downcast(
                "DynamicHandlerAdapter State".into(),
            ))
        })?;

        self.handler.handle(*event, *state)
    }
}

pub struct DynamicHandler<'a, M>
where
    M: std::fmt::Debug,
{
    event_type: EventDiscriminants,
    handler: Box<dyn EventHandler<M, Event = Box<dyn Any>, State = Box<dyn Any>> + 'a>,
}

impl<'a, M> EventHandler<M> for DynamicHandler<'a, M>
where
    M: std::fmt::Debug,
{
    type Event = Box<dyn Any>; // The adapted event type is now Box<dyn Any>
    type State = Box<dyn Any>;

    fn handle(&self, event: Self::Event, state: Self::State) -> Result<Task<M>, Error> {
        self.handler.handle(event, state)
    }
}

impl<'a, M> DynamicHandler<'a, M>
where
    M: std::fmt::Debug + 'a,
{
    pub fn new<E, S>(
        event_type: EventDiscriminants,
        handler: impl EventHandler<M, Event = E, State = S> + 'a, // Take an event handler for a specific event type E
    ) -> Self
    where
        E: Any,
        S: Any,
    {
        Self {
            event_type,
            // Use EventHandlerAdapter to convert the handler to one that accepts Box<dyn Any>
            handler: Box::new(DynamicHandlerAdapter::<'a, M, E, S>::new(handler))
                as Box<dyn EventHandler<M, Event = Box<dyn Any>, State = Box<dyn Any>>>,
        }
    }
}

impl<'a, M> std::fmt::Debug for DynamicHandler<'a, M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynamicHandler for {:?}", self.event_type)
    }
}
