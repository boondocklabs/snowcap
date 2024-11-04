//! File Module

use std::fs::Metadata;
use std::path::PathBuf;

use super::data::{ModuleData, ModuleDataKind};
use super::internal::ModuleInternal;
use super::{error::ModuleError, message::ModuleMessage, Module, ModuleEvent, ModuleInitData};
use crate::module::argument::ModuleArguments;
use async_trait::async_trait;
use file_format::FileFormat;
use iced::Task;
use tokio::fs::File;
use tokio::{fs, io::AsyncReadExt as _};

mod format;

pub struct FileContents {
    metadata: Metadata,
    buf: Vec<u8>,
    format: FileFormat,
}

impl ModuleData for FileContents {
    fn kind(&self) -> ModuleDataKind {
        ModuleDataKind::from(self.format)
    }

    fn bytes(&self) -> Result<&Vec<u8>, ModuleError> {
        Ok(&self.buf)
    }
}

impl std::fmt::Debug for FileContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileContents")
            .field("metadata", &self.metadata)
            .field("format", &self.format)
            .finish()
    }
}

#[derive(Debug)]
pub(super) enum FileEvent {
    Open(PathBuf),
    Opened(File),
    Loaded(FileContents),
}

impl ModuleEvent for FileEvent {}

#[derive(Default, Debug)]
pub(super) struct FileModule {
    path: Option<PathBuf>,
}

/// File module implementation
#[async_trait]
impl Module for FileModule {
    type Event = FileEvent;
    type Data = FileContents;

    async fn init(
        &mut self,
        args: ModuleArguments,
        _init_data: ModuleInitData,
    ) -> Result<Self::Event, ModuleError> {
        self.path = Some(args.get("path")?.to_string().into());

        // Return error if the file doesn't exist
        fs::try_exists(self.path.as_ref().unwrap()).await?;

        Ok(FileEvent::Open(self.path.clone().unwrap()))
    }

    fn on_event(&mut self, event: Self::Event) -> Task<ModuleMessage> {
        match event {
            FileEvent::Open(path) => Task::perform(
                async move {
                    let file = File::open(path).await?;
                    Ok(FileEvent::Opened(file))
                },
                |result: Result<FileEvent, crate::Error>| ModuleMessage::from(result),
            ),
            FileEvent::Opened(mut file) => Task::perform(
                async move {
                    let metadata = file.metadata().await?;

                    let mut buf = Vec::with_capacity(metadata.len() as usize);
                    let size = file.read_to_end(&mut buf).await?;
                    assert_eq!(size, metadata.len() as usize);

                    let contents = tokio::task::spawn_blocking(move || {
                        let format = FileFormat::from_bytes(&buf);
                        FileContents {
                            metadata,
                            buf,
                            format,
                        }
                    })
                    .await
                    .map_err(crate::Error::Tokio)?;

                    Ok(FileEvent::Loaded(contents))
                },
                |result: Result<FileEvent, crate::Error>| ModuleMessage::from(result),
            ),
            FileEvent::Loaded(contents) => self.send_data(contents),
        }
    }
}
