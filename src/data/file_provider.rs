use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use arbutus::NodeId;
use file_format::FileFormat;
use iced::Task;
use parking_lot::Mutex;
use tokio::io::AsyncReadExt;
use tracing::{error, info, info_span};

use crate::{connector::Inlet, message::Event, parser::error::ParseError, Error};

use super::{
    provider::{DynProvider, Provider, ProviderEvent},
    FileData,
};

#[derive(Debug)]
pub struct FileProvider {
    this: Option<Arc<Mutex<DynProvider>>>,
    path: PathBuf,
    inlet: Mutex<Option<Inlet<Event>>>,
    node_id: Option<NodeId>,
}

impl std::fmt::Display for FileProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileProvider path={}", self.path.display())
    }
}

impl FileProvider {
    pub fn new(filename: &Path) -> Result<Self, ParseError> {
        info!("FileProvider filename='{filename:?}'");

        let path: PathBuf = std::fs::canonicalize(filename)?.into();

        Ok(Self {
            this: None,
            path,
            node_id: None,
            inlet: Mutex::new(None),
        })
    }
}

impl FileProvider {
    async fn read_async(path: &PathBuf) -> Result<Vec<u8>, Error> {
        let mut f = tokio::fs::File::open(path).await?;
        let metadata = f.metadata().await?;

        info!("Opened file {path:?} length={}", metadata.len());

        let mut buf = Vec::with_capacity(metadata.len() as usize);
        f.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    fn process_file(path: &Path, bytes: Vec<u8>) -> Result<FileData, Error> {
        let result = file_format::FileFormat::from_bytes(&bytes);
        info!("Found file format {:?} {:?}", result.kind(), result);

        let data = match result.kind() {
            file_format::Kind::Archive => todo!(),
            file_format::Kind::Audio => todo!(),
            file_format::Kind::Compressed => todo!(),
            file_format::Kind::Database => todo!(),
            file_format::Kind::Diagram => todo!(),
            file_format::Kind::Disk => todo!(),
            file_format::Kind::Document => todo!(),
            file_format::Kind::Ebook => todo!(),
            file_format::Kind::Executable => todo!(),
            file_format::Kind::Font => todo!(),
            file_format::Kind::Formula => todo!(),
            file_format::Kind::Geospatial => todo!(),
            file_format::Kind::Image => {
                if FileFormat::ScalableVectorGraphics == result {
                    FileData::Svg(iced::widget::svg::Handle::from_memory(bytes))
                } else {
                    FileData::Image(iced::widget::image::Handle::from_bytes(bytes))
                }
            }
            file_format::Kind::Metadata => todo!(),
            file_format::Kind::Model => todo!(),
            file_format::Kind::Other => {
                if FileFormat::PlainText == result {
                    let string = String::from_utf8(bytes).map_err(Error::Encoding)?;

                    if let Some(extension) = path.extension() {
                        match extension.to_ascii_lowercase().to_str().unwrap() {
                            "md" => {
                                info!("Found Markdown extension");
                                let items =
                                    iced::widget::markdown::parse(string.as_str()).collect();
                                FileData::Markdown(items)
                            }
                            _ => FileData::Text(string),
                        }
                    } else {
                        FileData::Text(string)
                    }
                } else {
                    todo!();
                }
            }
            file_format::Kind::Package => todo!(),
            file_format::Kind::Playlist => todo!(),
            file_format::Kind::Presentation => todo!(),
            file_format::Kind::Rom => todo!(),
            file_format::Kind::Spreadsheet => todo!(),
            file_format::Kind::Subtitle => todo!(),
            file_format::Kind::Video => todo!(),
        };

        Ok(data)
    }
}

impl Provider for FileProvider {
    type H = crate::SnowHasher;

    fn set_event_inlet(&self, inlet: Inlet<Event>) {
        *self.inlet.lock() = Some(inlet);
    }

    fn update_task(&mut self) -> iced::Task<Event> {
        info_span!("FileProvider").in_scope(|| {
            let node_id = self.node_id;

            if None == node_id {
                error!("Node ID is not set for {self:?}");
                return Task::none();
            }

            let node_id = node_id.unwrap();

            let path = self.path.clone();
            Task::perform(
                async move {
                    let bytes = Self::read_async(&path).await?;
                    tokio::task::spawn_blocking(move || Self::process_file(&path, bytes))
                        .await
                        .map_err(Error::Tokio)?
                },
                move |res: Result<FileData, Error>| match res {
                    Ok(data) => Event::Provider(ProviderEvent::FileLoaded { node_id, data }),
                    Err(e) => Event::Provider(ProviderEvent::Error(e.to_string())),
                },
            )
        })
    }

    fn init_task(&mut self, this: Arc<Mutex<DynProvider>>, node_id: NodeId) -> iced::Task<Event> {
        info!("File Provider Init");

        self.this = Some(this.clone());

        let filename = self.path.clone();
        let task = Task::perform(async move { (this, filename) }, move |(this, filename)| {
            Event::WatchFileRequest {
                filename,
                provider: this,
            }
        });

        self.node_id = Some(node_id);
        task.chain(self.update_task())
    }

    fn set_node_id(&mut self, node_id: arbutus::NodeId) {
        self.node_id = Some(node_id);
    }

    fn hash_source(&self, hasher: &mut dyn std::hash::Hasher) {
        hasher.write(self.path.as_os_str().as_bytes());
    }
}
