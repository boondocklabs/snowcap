//! HTTP Request Module

use super::data::{ModuleData, ModuleDataKind};
use super::internal::ModuleInternal;
use super::{error::ModuleError, message::ModuleMessage, Module, ModuleEvent, ModuleInitData};
use crate::module::argument::ModuleArguments;
use crate::Value;
use async_trait::async_trait;
use iced::Task;
use reqwest::Url;
use reqwest::{header, Client, Method};
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum HttpError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug)]
pub(super) enum HttpEvent {
    StartRequest,
    Request(reqwest::Request),
    Response(reqwest::Response),
    Data(HttpData),
}

pub struct HttpData {
    url: Url,
    kind: ModuleDataKind,
    data: Vec<u8>,
}

impl std::fmt::Debug for HttpData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpData")
            .field("url", &self.url)
            .field("kind", &self.kind)
            .field("length", &self.data.len())
            .finish()
    }
}

impl ModuleData for HttpData {
    fn kind(&self) -> ModuleDataKind {
        self.kind
    }

    fn bytes(&self) -> Result<&Vec<u8>, ModuleError> {
        Ok(&self.data)
    }
}

impl ModuleEvent for HttpEvent {}

#[derive(Default, Debug)]
pub(super) struct HttpModule {
    method: Option<Method>,
    url: Option<Url>,
    client: Option<Client>,
}

#[async_trait]
impl Module for HttpModule {
    type Event = HttpEvent;
    type Data = HttpData;
    async fn init(
        &mut self,
        args: ModuleArguments,
        _init_data: ModuleInitData,
    ) -> Result<Self::Event, ModuleError> {
        let method = args
            .get("method")
            .unwrap_or(&Value::new_string("get".into()))
            .clone();

        match Method::from_bytes(method.to_string().to_uppercase().as_bytes()) {
            Ok(method) => self.method = Some(method),
            Err(e) => {
                return Err(ModuleError::InvalidArgument(format!(
                    "Failed to parse Method: {e:?}"
                )));
            }
        }

        let url = args.get("url")?.to_string();
        match Url::parse(url.as_str()) {
            Ok(url) => self.url = Some(url),
            Err(e) => {
                return Err(ModuleError::InvalidArgument(format!(
                    "Failed to parse URL: {e:?}"
                )));
            }
        }

        self.client = Some(
            reqwest::ClientBuilder::new()
                .connection_verbose(true)
                .user_agent("Snowcap")
                .build()
                .map_err(|e| ModuleError::Internal(Box::new(e)))?,
        );

        Ok(HttpEvent::StartRequest)
    }

    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessage> {
        match event {
            HttpEvent::StartRequest => {
                let client = self.client.as_ref().unwrap().clone();
                let method = self.method.as_ref().unwrap().clone();
                let url = self.url.as_ref().unwrap().clone();

                Task::perform(
                    async move {
                        let req = client
                            .request(method, url)
                            .header(header::ACCEPT, "*/*")
                            .build()?;
                        Ok(HttpEvent::Request(req))
                    },
                    |result: Result<HttpEvent, HttpError>| ModuleMessage::from(result),
                )
            }

            HttpEvent::Request(request) => {
                let client = self.client.as_ref().unwrap().clone();
                Task::perform(
                    async move {
                        let response = client.execute(request).await?;

                        Ok(HttpEvent::Response(response))
                    },
                    |result: Result<HttpEvent, HttpError>| ModuleMessage::from(result),
                )
            }

            HttpEvent::Response(response) => match response.headers().get(header::CONTENT_TYPE) {
                Some(content_type) => {
                    let url = self.url.clone().unwrap();

                    debug!("Content Type: {content_type:?}");

                    let mime: mime::Mime = content_type.to_str().unwrap().parse().unwrap();

                    match mime.type_() {
                        mime::IMAGE => Task::perform(
                            async move {
                                let bytes = response.bytes().await.map_err(HttpError::Reqwest)?;

                                let data = HttpData {
                                    url,
                                    kind: ModuleDataKind::Image,
                                    data: bytes.to_vec(),
                                };

                                Ok(HttpEvent::Data(data))
                            },
                            |result: Result<HttpEvent, HttpError>| ModuleMessage::from(result),
                        ),
                        mime::TEXT => Task::perform(
                            async move {
                                let text = response.text().await.map_err(HttpError::Reqwest)?;

                                let data = HttpData {
                                    url,
                                    kind: ModuleDataKind::Text,
                                    data: text.as_bytes().to_vec(),
                                };

                                Ok(HttpEvent::Data(data))
                            },
                            |result: Result<HttpEvent, HttpError>| ModuleMessage::from(result),
                        ),
                        _ => {
                            error!("Unknown content type {content_type:?}");
                            Task::none()
                        }
                    }
                }
                None => {
                    error!("Content-type not provided in response");
                    Task::none()
                }
            },

            HttpEvent::Data(data) => self.send_data(data),
        }
    }

    fn on_message(&mut self, message: ModuleMessage) -> Task<ModuleMessage> {
        println!("HTTP on_message {message:#?}");
        Task::none()
    }
}
