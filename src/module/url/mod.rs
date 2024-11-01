use async_trait::async_trait;
use iced::Task;

use super::{
    message::ModuleMessageKind, Module, ModuleAsync, ModuleAsyncInitData, ModuleEvent, ModuleInit,
};

#[derive(Debug)]
pub enum UrlEvent {
    Init(String),
    Loaded(String),
}

impl ModuleEvent for UrlEvent {}

#[derive(Default, Debug)]
pub struct UrlModule {
    data: String,
}

impl UrlModule {
    async fn init() {}
}

impl ModuleInit for UrlModule {}

#[async_trait]
impl ModuleAsync for UrlModule {
    type Event = UrlEvent;
    async fn init(&mut self, init_data: ModuleAsyncInitData) -> Self::Event {
        UrlEvent::Init("Url Init".into())
    }
}

impl Module for UrlModule {
    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessageKind> {
        println!("Received event from ourselves: {event:?}");
        match event {
            UrlEvent::Init(data) => self.data = data,
            UrlEvent::Loaded(_) => todo!(),
        }

        Task::done(ModuleMessageKind::Debug("on_event done!"))
    }
}
