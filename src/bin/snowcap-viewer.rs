use iced::{Element, Task, Theme};
use snowcap::Snowcap;
use tracing_subscriber::{self, EnvFilter};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();

    let filename = args[1].clone();

    iced::application("Snowcap", SnowcapViewer::update, SnowcapViewer::view)
        .theme(SnowcapViewer::theme)
        .run_with(move || {
            let mut viewer = SnowcapViewer::new(filename.clone());
            let init_task = viewer.init();
            (
                // Provide initial state and init task
                viewer, init_task,
            )
        })
}

struct SnowcapViewer {
    filename: String,
    parse_error: Option<snowcap::Error>,
    snow: Option<Snowcap<Message>>,
}

impl SnowcapViewer {
    pub fn new(filename: String) -> Self {
        let mut viewer = Self {
            filename,
            snow: None,
            parse_error: None,
        };
        viewer.load().ok();
        viewer
    }

    pub fn load(&mut self) -> Result<(), snowcap::Error> {
        let mut snow = Snowcap::new()?;
        snow.load_file(self.filename.clone())?;
        self.snow = Some(snow);

        Ok(())
    }

    fn init(&mut self) -> Task<snowcap::Message<Message>> {
        if let Some(snow) = &mut self.snow {
            return snow.init();
        }
        Task::none()
    }
}

#[derive(Debug, Clone)]
enum Message {}

impl SnowcapViewer {
    fn update(
        &mut self,
        mut message: snowcap::Message<Message>,
    ) -> Task<snowcap::Message<Message>> {
        if let Some(snow) = &mut self.snow {
            snow.update(&mut message)
        } else {
            Task::none()
        }

        /*
        match message {
            snowcap::Message::App(app) => match app {
                Message::Watcher(event) => {
                    debug!("Watcher {event:?}");
                    match event.kind {
                        notify::EventKind::Modify(ModifyKind::Data(_)) => {
                            info!("Snowcap File Modified. Reloading");
                            self.load().ok();
                            Task::none()
                        }
                        _ => Task::none(),
                    }
                }
            },
        }
        */
    }

    fn view(&self) -> Element<snowcap::Message<Message>> {
        if let Some(snow) = &self.snow {
            snow.view()
        } else if let Some(err) = &self.parse_error {
            iced::widget::text(format!("{err:#?}")).into()
        } else {
            iced::widget::text("No snowcap file loaded").into()
        }
    }

    fn theme(&self) -> Theme {
        //Theme::TokyoNight
        Theme::CatppuccinFrappe
    }
}
