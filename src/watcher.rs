//! Filesystem Watcher

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{
    futures::{self, channel::mpsc, SinkExt as _},
    Task,
};
use notify::{FsEventWatcher, RecommendedWatcher, Watcher as _};
use tracing::{debug, info, info_span, instrument, warn, Instrument as _};

use crate::Error;

use crate::Message;

#[derive(Debug)]
pub struct FileWatcher {
    watcher: FsEventWatcher,
    watched_paths: HashSet<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum WatchMessage {
    None,
    Command(WatchCommand),
    Event(WatchEvent),
}

#[derive(Debug, Clone)]
pub enum WatchCommand {}

#[derive(Debug, Clone)]
pub enum WatchEvent {
    Test,
    Error(Arc<Box<dyn std::error::Error + Send + Sync>>),
}

/// Internal messages sent between [`notify`] and an [`iced::Task`] over an mpsc channel
#[derive(Debug)]
enum InternalMessage {
    /// Underlying event from [`notify`] sent over internal mpsc channel to Task
    Event(notify::Event),

    /// Error from [`notify`]
    Error(notify::Error),
}

impl FileWatcher {
    pub fn new() -> (Self, Task<Message>) {
        let (mut tx, rx) = mpsc::channel(1024);

        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                let message = match res {
                    Ok(fsevent) => InternalMessage::Event(fsevent),
                    Err(e) => InternalMessage::Error(e),
                };

                futures::executor::block_on(
                    async {
                        debug!("Sending {message:?}");
                        tx.send(message).await.unwrap();
                    }
                    .instrument(info_span!("watcher")),
                )
            },
            notify::Config::default(),
        )
        .unwrap();

        let task: Task<Message> = Task::run(rx, |msg| {
            info_span!("watcher").in_scope(|| {
                debug!("{msg:?}");

                let watch_message = match msg {
                    InternalMessage::Event(event) => match event.kind {
                        notify::EventKind::Modify(notify::event::ModifyKind::Data(change)) => {
                            info!("Data Modified: '{:?}' paths: {:?}", change, event.paths);
                            WatchMessage::Event(WatchEvent::Test)
                        }

                        _ => {
                            warn!("Unhandled notify event {event:#?}");
                            WatchMessage::None
                        }
                    },
                    InternalMessage::Error(error) => {
                        WatchMessage::Event(WatchEvent::Error(Arc::new(Box::new(error))))
                    }
                };

                Message::broadcast(watch_message)
            })
        });

        (
            Self {
                watcher,
                watched_paths: HashSet::new(),
            },
            task,
        )
    }

    /// Add a path to monitor to the [`FileWatcher`]
    #[instrument(name = "watcher")]
    pub fn watch(&mut self, path: &Path) -> Result<(), Error> {
        if self.watched_paths.insert(path.into()) {
            debug!("Added '{path:?}'");

            return self
                .watcher
                .watch(path, notify::RecursiveMode::NonRecursive)
                .map_err(Error::Notify);
        }

        warn!("Adding duplicate path '{path:?}' to the watcher");

        Ok(())
    }

    /// Remove a path from the [`FileWatcher`]
    #[instrument(name = "watcher")]
    pub fn unwatch(&mut self, path: &Path) -> Result<(), Error> {
        if self.watched_paths.remove(path) {
            debug!("Removed '{path:?}'");
            return self.watcher.unwatch(path).map_err(Error::Notify);
        }

        warn!("Removing path '{path:?}' which does not exist in the watcher");

        Ok(())
    }
}
