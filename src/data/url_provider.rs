use std::{sync::Arc, thread::sleep, time::Duration};

use super::{
    provider::{DynProvider, Provider, ProviderEvent},
    FileData,
};
use crate::{connector::Inlet, message::Event, parser::error::ParseError};
use arbutus::NodeId;
use colored::Colorize;
use iced::Task;
use parking_lot::Mutex;
use reqwest::header::CONTENT_TYPE;
use tracing::{debug, info, warn};
use url::Url;

#[derive(Debug)]
pub struct UrlProvider {
    url: Url,
    node_id: Option<NodeId>,
    inlet: Mutex<Option<Inlet<Event>>>,
}

impl Drop for UrlProvider {
    fn drop(&mut self) {
        println!("{}", "URL PROVIDER DROPPED".bright_magenta());
    }
}

impl std::fmt::Display for UrlProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UrlProvider url={}", self.url)
    }
}

impl UrlProvider {
    pub fn new(url: &str) -> Result<Self, ParseError> {
        let url = Url::parse(url)?;

        info!("Created URL Provider for {url:?}");

        Ok(Self {
            url,
            node_id: None,
            inlet: Mutex::new(None),
        })
    }
}

impl Provider for UrlProvider {
    type H = crate::SnowHasher;

    fn set_event_inlet(&self, inlet: Inlet<Event>) {
        *self.inlet.lock() = Some(inlet)
    }

    fn update_task(&self) -> iced::Task<Event> {
        let url = self.url.clone();
        let node_id = self.node_id.unwrap().clone();
        Task::perform(
            async move {
                loop {
                    match reqwest::get(url.clone()).await {
                        Ok(response) => {
                            let data =
                                if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                                    debug!("Content Type: {content_type:?}");
                                    let mime: mime::Mime =
                                        content_type.to_str().unwrap().parse().unwrap();

                                    match mime.type_() {
                                        mime::IMAGE => {
                                            let bytes = response.bytes().await.unwrap();

                                            FileData::Image(
                                                iced::widget::image::Handle::from_bytes(bytes),
                                            )
                                        }
                                        mime::TEXT => {
                                            let text = response.text().await.unwrap();
                                            FileData::Text(text)
                                        }
                                        _ => FileData::Text("Unknown content type".into()),
                                    }
                                } else {
                                    FileData::Text("Unknown content type".into())
                                };
                            return (node_id, url, data);
                        }

                        Err(e) => {
                            warn!("{e:?}")
                        }
                    }

                    sleep(Duration::from_secs(1));
                }
            },
            |(node_id, url, data)| Event::Provider(ProviderEvent::UrlLoaded { node_id, url, data }),
        )
    }

    fn init_task(&mut self, _this: Arc<Mutex<DynProvider>>, node_id: NodeId) -> Task<Event> {
        self.node_id = Some(node_id);

        Task::perform(
            async move { tracing::info!("UrlProvider Init Task") },
            |_| Event::Provider(ProviderEvent::Initialized),
        )
        .chain(self.update_task())
    }

    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = Some(node_id);
    }
    fn hash_source(&self, hasher: &mut dyn std::hash::Hasher) {
        hasher.write(self.url.as_str().as_bytes());
    }
}
