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
pub struct HttpModule {
    data: String,
}

impl ModuleInit for HttpModule {}

#[async_trait]
impl ModuleAsync for HttpModule {
    type Event = UrlEvent;
    async fn init(&mut self, init_data: ModuleAsyncInitData) -> Self::Event {
        UrlEvent::Init("HTTP Module Async Init".into())
    }
}

impl Module for HttpModule {
    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessageKind> {
        println!("Received event from ourselves: {event:?}");
        match event {
            UrlEvent::Init(data) => self.data = data,
            UrlEvent::Loaded(_) => todo!(),
        }

        Task::done(ModuleMessageKind::Debug("on_event done!"))
    }

    fn on_message(&mut self, _message: ModuleMessageKind) -> Task<ModuleMessageKind> {
        println!("HTTP on_message!");
        Task::none()
    }
}
