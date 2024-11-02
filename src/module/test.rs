use async_trait::async_trait;
use iced::Task;
use tracing::debug;

use crate::parser::module::ModuleArguments;

use super::{
    message::ModuleMessageKind, Module, ModuleAsync, ModuleAsyncInitData, ModuleEvent, ModuleInit,
};

#[derive(Debug)]
pub enum FooEvent {
    Init(String),
}
impl ModuleEvent for FooEvent {}

#[derive(Default, Debug)]
pub struct Foo {
    data: String,
}
impl ModuleInit for Foo {}

#[async_trait]
impl ModuleAsync for Foo {
    type Event = FooEvent;
    async fn init(
        &mut self,
        args: ModuleArguments,
        _init_data: ModuleAsyncInitData,
    ) -> Self::Event {
        debug!("Test module init");
        FooEvent::Init("Hello World".into())
    }
}

impl Module for Foo {
    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessageKind> {
        println!("Received event from ourselves: {event:?}");
        match event {
            FooEvent::Init(data) => self.data = data,
        }

        Task::future(async move {
            println!("I am a test task run after processing on_event handler");
            ModuleMessageKind::None
        })
    }
}
