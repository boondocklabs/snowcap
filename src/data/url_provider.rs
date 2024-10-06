use std::sync::Arc;

use super::{
    file_provider::FileData,
    provider::{Provider, ProviderEvent},
};
use crate::{
    connector::Inlet,
    message::Event,
    parser::{error::ParseError, NodeId},
};
use iced::Task;
use parking_lot::Mutex;
use reqwest::header::CONTENT_TYPE;
use tracing::info;
use url::Url;

#[derive(Debug)]
pub struct UrlProvider {
    url: Url,
    node_id: Option<NodeId>,
    inlet: Mutex<Option<Inlet<Event>>>,
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
    fn set_event_inlet(&self, inlet: Inlet<Event>) {
        *self.inlet.lock() = Some(inlet)
    }

    fn update_task(&self) -> iced::Task<Event> {
        let url = self.url.clone();
        let node_id = self.node_id.unwrap().clone();
        Task::perform(
            async move {
                let response = reqwest::get(url.clone()).await.unwrap();

                let data = if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                    info!("CONTENT TYPE {content_type:?}");
                    let mime: mime::Mime = content_type.to_str().unwrap().parse().unwrap();

                    match mime.type_() {
                        mime::IMAGE => {
                            let bytes = response.bytes().await.unwrap();

                            FileData::Image(iced::widget::image::Handle::from_bytes(bytes))
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

                (node_id, url, data)
            },
            |(node_id, url, data)| {
                Event::Provider(ProviderEvent::UrlLoaded {
                    node_id,
                    url,
                    data: data,
                })
            },
        )
    }

    fn init_task(&mut self, _this: Arc<Mutex<dyn Provider>>, node_id: NodeId) -> Task<Event> {
        self.node_id = Some(node_id);
        self.update_task()
    }

    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = Some(node_id);
    }
}
