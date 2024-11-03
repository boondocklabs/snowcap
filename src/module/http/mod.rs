use super::{error::ModuleError, message::ModuleMessage, Module, ModuleEvent, ModuleInitData};
use crate::module::argument::ModuleArguments;
use crate::Value;
use async_trait::async_trait;
use iced::advanced::image::Bytes;
use iced::Task;
use reqwest::Url;
use reqwest::{header, Client, Method};
use tracing::error;

#[derive(Debug)]
pub(super) enum HttpEvent {
    Request(Request),
    Response(reqwest::Response),
    Error(reqwest::Error),
    Bytes(Bytes),
}

impl ModuleEvent for HttpEvent {}

#[derive(Default, Debug)]
pub(super) struct HttpModule {
    method: Option<Method>,
    url: Option<Url>,
    client: Option<Client>,
}

impl HttpModule {
    async fn request(request: Request) -> Result<reqwest::Response, reqwest::Error> {
        let req = request
            .client
            .request(request.method, request.url)
            .header(header::ACCEPT, "*/*")
            .build()?;

        let response = request.client.execute(req).await?;

        Ok(response)
    }
}

#[derive(Debug)]
pub(super) struct Request {
    client: Client,
    method: Method,
    url: Url,
}

#[async_trait]
impl Module for HttpModule {
    type Event = HttpEvent;
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

        match reqwest::ClientBuilder::new()
            .connection_verbose(true)
            .user_agent("Snowcap")
            .build()
        {
            Ok(client) => {
                self.client = Some(client);

                // Create a request object, and start the request
                let request = Request {
                    client: self.client.as_ref().unwrap().clone(),
                    method: self.method.as_ref().unwrap().clone(),
                    url: self.url.as_ref().unwrap().clone(),
                };

                Ok(HttpEvent::Request(request))
            }
            Err(err) => Ok(HttpEvent::Error(err)),
        }
    }

    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessage> {
        match event {
            HttpEvent::Request(request) => Task::perform(
                async move {
                    Self::request(request)
                        .await
                        .map_or_else(|err| HttpEvent::Error(err), |res| HttpEvent::Response(res))
                },
                |event| ModuleMessage::from(event),
            ),

            HttpEvent::Response(response) => {
                match response.headers().get(header::CONTENT_TYPE) {
                    Some(_content_type) => {}
                    None => {
                        //Task::done(ModuleMessageKind::from(HttpEvent::Error(I/)))
                    }
                }

                Task::perform(
                    async move {
                        response.bytes().await.map_or_else(
                            |err| HttpEvent::Error(err),
                            |bytes| HttpEvent::Bytes(bytes),
                        )
                    },
                    |event| ModuleMessage::from(event),
                )
            }

            HttpEvent::Bytes(bytes) => {
                tracing::info!("Received Bytes: {bytes:#?}");

                Task::none()
            }

            HttpEvent::Error(error) => {
                error!("{error:?}");
                Task::none()
            }
        }
    }

    fn on_message(&mut self, message: ModuleMessage) -> Task<ModuleMessage> {
        println!("HTTP on_message {message:#?}");
        Task::none()
    }
}
