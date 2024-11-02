use std::time::Duration;

use async_trait::async_trait;
use iced::Task;
use tokio::time::Instant;
use tokio_stream::wrappers::IntervalStream;
use tracing::{debug, error};

use crate::{parser::module::ModuleArguments, NodeRef};

use super::{
    message::{Channel, ChannelData, ModuleMessageKind},
    Module, ModuleAsync, ModuleAsyncInitData, ModuleEvent, ModuleInit,
};

#[derive(Debug)]
pub enum TimingEvent {
    Init(IntervalStream),
    Tick(Instant),
    Failed,
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
    async fn init(
        &mut self,
        args: ModuleArguments,
        _init_data: ModuleAsyncInitData,
    ) -> Self::Event {
        debug!("Timing module init");

        debug!("Arguments: {}", args);

        if let Some(interval) = args.get("interval") {
            let interval: &String = interval.into();
            match duration_str::parse(interval) {
                Ok(duration) => {
                    let interval = tokio::time::interval(duration);
                    let stream = IntervalStream::new(interval);

                    return TimingEvent::Init(stream);
                }
                Err(e) => {
                    error!("Failed to convert interval argument");
                }
            }
        }

        TimingEvent::Failed
    }
}

impl Module for TimingModule {
    fn init_tree(&mut self, tree: Option<&NodeRef<Self::Event>>) {
        debug!("Initialize tree in Timing module: {tree:#?}");
    }
    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessageKind> {
        match event {
            TimingEvent::Init(stream) => Task::done(ModuleMessageKind::Subscribe(Channel("tick")))
                .chain(Task::run(stream, |_instant| {
                    ModuleMessageKind::Publish(super::message::PublishMessage {
                        channel: Channel("tick"),
                        data: ChannelData::Trigger,
                    })
                })),
            TimingEvent::Tick(instant) => {
                println!("Timing Module: TICK {instant:?}");
                Task::none()
            }
            TimingEvent::Failed => {
                println!("Timing module failed event");
                Task::none()
            }
        }
    }

    fn on_message(&mut self, message: ModuleMessageKind) -> Task<ModuleMessageKind> {
        println!("Module received message {message:?}");
        Task::none()
    }
}
