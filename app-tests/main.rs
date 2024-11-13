use std::time::Duration;

use iced::Task;
use snowcap::{message::Command, Message, Snowcap};
use tokio::time::sleep;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
enum TestMessage {
    // Application context is ready to begin test
    Ready,
}

fn harness(markup: &'static str) {
    let mut window = iced::window::Settings::default();
    window.visible = false;

    iced::application("Test", Snowcap::update, Snowcap::view)
        .window(window)
        .run_with(|| {
            let mut snow = Snowcap::new().unwrap();
            let init_tasks = snow.init();

            let exit_task = Task::future(async move {
                println!("Sleeping");
                sleep(Duration::from_secs(2)).await;
                Message::broadcast(Command::Shutdown)
            });

            let tasks = Task::batch([
                init_tasks,
                exit_task,
                Task::done(Message::broadcast(TestMessage::Ready)),
            ]);

            snow.router().static_endpoint::<TestMessage, _>(|s, msg| {
                println!("TEST MESSAGE {msg:#?}");
                Task::none()
            });

            if let Err(e) = snow.load_memory(markup) {
                println!("{}", e);
                return (snow, iced::exit());
            }

            (snow, tasks)
        })
        .unwrap();
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("salish=trace"))
        //.with_file(true)
        //.with_line_number(true)
        .init();

    //snow.load_memory(r#"{text("Hello")}"#).unwrap();
    harness(r#"{text(http!{url:"http://icanhazip.com"})}"#);
}
