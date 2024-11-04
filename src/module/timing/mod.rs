use async_trait::async_trait;
use iced::Task;
use tokio::time::Instant;
use tokio_stream::wrappers::IntervalStream;
use tracing::{debug, error};

use crate::{module::argument::ModuleArguments, NodeRef};

use super::{
    data::ModuleData,
    error::ModuleError,
    message::{ModuleMessage, Topic, TopicMessage},
    Module, ModuleEvent, ModuleInitData,
};

#[derive(Debug)]
pub struct TimingData;
impl ModuleData for TimingData {
    fn kind(&self) -> super::data::ModuleDataKind {
        todo!()
    }

    fn bytes(&self) -> Result<&Vec<u8>, ModuleError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum TimingEvent {
    Init(IntervalStream),
    Tick(Instant),
    Failed,
}
impl ModuleEvent for TimingEvent {}

#[derive(Default, Debug)]
pub struct TimingModule {}

#[async_trait]
impl Module for TimingModule {
    type Event = TimingEvent;
    type Data = TimingData;

    async fn init(
        &mut self,
        args: ModuleArguments,
        _init_data: ModuleInitData,
    ) -> Result<Self::Event, ModuleError> {
        debug!("Timing module init");

        /*
        let interval = args
            .get("interval")
            .unwrap_or(&Value::new_string("1s".into()));
        */
        let interval = args.get("interval")?;

        let interval: String = interval.to_string();
        match duration_str::parse(interval) {
            Ok(duration) => {
                let interval = tokio::time::interval(duration);
                let stream = IntervalStream::new(interval);

                Ok(TimingEvent::Init(stream))
            }
            Err(e) => {
                error!("Failed to convert interval argument");
                Err(ModuleError::InvalidArgument(format!(
                    "Cannot parse interval: '{e}'"
                )))
            }
        }
    }

    fn init_tree(&mut self, tree: Option<&NodeRef<Self::Event>>) {
        debug!("Initialize tree in Timing module: {tree:#?}");
    }

    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessage> {
        match event {
            TimingEvent::Init(stream) => Task::done(ModuleMessage::Subscribe(Topic("tick"))).chain(
                Task::run(stream, |_instant| {
                    ModuleMessage::Publish(super::message::PublishMessage {
                        topic: Topic("tick"),
                        message: TopicMessage::Trigger,
                    })
                }),
            ),
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

    fn on_message(&mut self, _message: ModuleMessage) -> Task<ModuleMessage> {
        Task::none()
    }

    fn on_subscription(&mut self, _topic: Topic, _message: TopicMessage) -> Task<ModuleMessage> {
        Task::none()
    }
}
