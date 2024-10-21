use std::process::exit;

use iced::{Element, Task, Theme};
use snowcap::Snowcap;
use tracing_subscriber::{self, EnvFilter};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .init();

    let args: Vec<String> = std::env::args().collect();

    let filename = args[1].clone();

    iced::application("Snowcap", SnowcapViewer::update, SnowcapViewer::view)
        .theme(SnowcapViewer::theme)
        .run_with(move || {
            let viewer = SnowcapViewer::new(filename.clone());

            match viewer {
                Ok(mut viewer) => {
                    let init_task = viewer.init();
                    (
                        // Provide initial state and init task
                        viewer, init_task,
                    )
                }
                Err(err) => {
                    tracing::error!("{:#?}", err);
                    exit(-1)
                }
            }
        })
}

struct SnowcapViewer {
    filename: String,
    snow: Snowcap<Message>,
}

impl SnowcapViewer {
    pub fn new(filename: String) -> Result<Self, snowcap::Error> {
        let mut viewer = Self {
            filename,
            snow: Snowcap::new().unwrap(),
        };

        viewer.load()?;

        Ok(viewer)
    }

    pub fn load(&mut self) -> Result<(), snowcap::Error> {
        self.snow.load_file(self.filename.clone())?;
        Ok(())
    }

    fn init(&mut self) -> Task<snowcap::Message<Message>> {
        self.snow.init()
    }
}

#[derive(Debug, Clone)]
enum Message {}

impl SnowcapViewer {
    fn update(
        &mut self,
        mut message: snowcap::Message<Message>,
    ) -> Task<snowcap::Message<Message>> {
        self.snow.update(&mut message)
    }

    fn view(&self) -> Element<snowcap::Message<Message>> {
        self.snow.view()
    }

    fn theme(&self) -> Theme {
        //Theme::TokyoNight
        Theme::CatppuccinFrappe
    }
}
