use iced::Task;
use salish::{message::Message, router::MessageRouter};
use tracing::debug;
use tracing_test::traced_test;

use crate::{
    module::{argument::ModuleArguments, manager::ModuleManager},
    Source,
};

#[traced_test]
#[test]
fn message() {
    let mut router = MessageRouter::<Task<Message>, Source>::new();

    let mut manager = ModuleManager::new(router.clone());

    let args = ModuleArguments::new().arg("url", r#""http://icanhazip.com""#);
    let (_mid, _task) = manager.instantiate(&"http".into(), args).unwrap();

    router.handle_message(Message::broadcast(0));

    debug!("{manager:#?}");
}
