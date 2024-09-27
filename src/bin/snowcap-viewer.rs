use std::fs;

use iced::{
    futures::{self, channel::mpsc::channel, SinkExt},
    Element, Task,
};
use notify::{event::ModifyKind, RecommendedWatcher, Watcher};
use snowcap::{Message, Snowcap, SnowcapParser};
use tracing::{debug, error, info};
use tracing_subscriber;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    let filename = args[1].clone();

    let (mut tx, rx) = channel(1);
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        notify::Config::default(),
    )
    .unwrap();

    watcher
        .watch(filename.as_ref(), notify::RecursiveMode::Recursive)
        .unwrap();

    iced::application("Snowcap", SnowcapViewer::update, SnowcapViewer::view).run_with(move || {
        let viewer = SnowcapViewer::new(filename.clone());
        (
            viewer,
            Task::run(rx, |event| Message::Watcher(event.unwrap())),
        )
    })
}

struct SnowcapViewer {
    filename: String,
    parse_error: Option<snowcap::Error>,
    root: Option<Snowcap>,
}

impl SnowcapViewer {
    pub fn new(filename: String) -> Self {
        let mut viewer = Self {
            filename,
            root: None,
            parse_error: None,
        };
        viewer.load().ok();
        viewer
    }

    pub fn load(&mut self) -> Result<(), snowcap::Error> {
        let data = fs::read_to_string(&self.filename).expect("cannot read file");

        match SnowcapParser::parse_file(&data) {
            Ok(root) => {
                self.root = Some(root);
                debug!("{:#?}", self.root);
                self.parse_error = None;
            }
            Err(e) => {
                self.root = None;
                self.parse_error = Some(e);
            }
        }

        Ok(())
    }
}

/*
#[derive(Debug, Clone)]
enum Message {
    Watcher(notify::Event),
}
*/

impl SnowcapViewer {
    fn update(&mut self, message: Message) {
        match message {
            Message::Watcher(event) => {
                debug!("Watcher {event:?}");
                match event.kind {
                    notify::EventKind::Modify(ModifyKind::Data(_)) => {
                        info!("Snowcap File Modified. Reloading");
                        self.load().ok();
                    }
                    _ => {}
                }
            }

            _ => {
                info!("Unhandled message {message:?}")
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if let Some(root) = &self.root {
            match root.root().try_into() {
                Ok(content) => content,
                Err(e) => {
                    error!("{e:?}");
                    iced::widget::text(format!("{e:?}")).into()
                }
            }
        } else if let Some(err) = &self.parse_error {
            iced::widget::text(format!("{err:#?}")).into()
        } else {
            iced::widget::text("No snowcap file loaded").into()
        }
    }
}
