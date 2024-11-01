use std::time::Duration;

use async_trait::async_trait;
use iced::Task;
use tokio::time::Instant;
use tokio_stream::wrappers::IntervalStream;
use tracing::debug;

use crate::NodeRef;

use super::{
    message::ModuleMessageKind, Module, ModuleAsync, ModuleAsyncInitData, ModuleEvent, ModuleInit,
};

#[derive(Debug)]
pub enum TimingEvent {
    Init(IntervalStream),
    Tick(Instant),
}
impl ModuleEvent for TimingEvent {}

#[derive(Default, Debug)]
pub struct TimingModule {
    stream: Option<IntervalStream>,
}
impl ModuleInit for TimingModule {}

#[async_trait]
impl ModuleAsync for TimingModule {
    type Event = TimingEvent;
    async fn init(&mut self, _init_data: ModuleAsyncInitData) -> Self::Event {
        debug!("Timing module init");

        let interval = tokio::time::interval(Duration::from_millis(1000));
        let stream = IntervalStream::new(interval);

        TimingEvent::Init(stream)
    }
}

impl Module for TimingModule {
    fn init_tree(&mut self, tree: Option<&NodeRef<Self::Event>>) {
        debug!("Initialize tree in Timing module: {tree:#?}");
    }
    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessageKind> {
        //println!("Received {event:?}");
        match event {
            TimingEvent::Init(stream) => {
                //self.stream = Some(data);

                Task::run(stream, |instant| {
                    ModuleMessageKind::from(TimingEvent::Tick(instant))
                })
            }
            TimingEvent::Tick(instant) => {
                println!("Timing Module: TICK {instant:?}");
                Task::none()
            }
        }
    }
}
