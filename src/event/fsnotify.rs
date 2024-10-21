use std::{collections::HashMap, path::PathBuf, sync::Arc};

use iced::{advanced::graphics::futures::MaybeSend, Task};
use parking_lot::Mutex;
use tracing::info;

use crate::{
    data::provider::DynProvider,
    message::{Command, Event},
    IndexedTree,
};

use super::EventHandler;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct FsNotifyState<M>
where
    M: std::fmt::Debug + 'static,
{
    tree: Arc<Mutex<Option<IndexedTree<M>>>>,
    pub provider_map: HashMap<PathBuf, Arc<Mutex<DynProvider>>>,
}

impl<M> FsNotifyState<M>
where
    M: std::fmt::Debug + 'static,
{
    pub fn new(tree: Arc<Mutex<Option<IndexedTree<M>>>>) -> Self {
        Self {
            tree,
            provider_map: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
pub struct FsNotifyEventHandler<M>
where
    M: std::fmt::Debug + 'static,
{
    tree: Arc<Mutex<Option<IndexedTree<M>>>>,
}

impl<M> FsNotifyEventHandler<M>
where
    M: std::fmt::Debug + 'static,
{
    pub fn new(tree: Arc<Mutex<Option<IndexedTree<M>>>>) -> Self {
        Self { tree }
    }
}

impl<M> EventHandler<M> for FsNotifyEventHandler<M>
where
    M: std::fmt::Debug + From<Event> + From<Command> + MaybeSend + 'static,
{
    type Event = notify::Event;
    type State = Arc<Mutex<FsNotifyState<M>>>;

    fn handle(
        &self,
        event: Self::Event,
        state: Self::State,
    ) -> Result<iced::Task<M>, crate::Error> {
        info!("FsNotify event handler");

        match event.kind {
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_change)) => {
                let mut tasks: Vec<Task<M>> = Vec::new();
                for path in &event.paths {
                    info!("File change notification for {path:?}");

                    // Find the provider of this file path from the provider map
                    if let Some(provider) = state.lock().provider_map.get(path) {
                        // Get the update task for this Provider
                        let task = provider.lock().update_task().map(|e| M::from(e));
                        tasks.push(task);
                    } else {
                        // Since we didn't find the path in the map of nodes
                        // which reference the changed file, we can assume
                        // that this is the markup file itself that has changed.

                        tasks.push(Task::done(M::from(Command::Reload)));
                    }
                }
                Ok(Task::batch(tasks))
            }
            _ => Ok(Task::none()),
        }
    }
}
