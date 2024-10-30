use std::{marker::PhantomData, sync::Arc, time::Duration};

use arbutus::TreeNodeRef as _;

use iced::Task;
use parking_lot::Mutex;
use tracing::debug;

use crate::{
    data::{provider::ProviderEvent, DataType, FileData, MarkdownItems},
    parser::value::ValueKind,
    Error, IndexedTree, NodeId, SyncError,
};

use super::EventHandler;
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ProviderState<M>
where
    M: std::fmt::Debug + 'static,
{
    tree: Arc<Mutex<Option<IndexedTree<M>>>>,
}

impl<M> ProviderState<M>
where
    M: std::fmt::Debug + 'static,
{
    pub fn new(tree: Arc<Mutex<Option<IndexedTree<M>>>>) -> Self {
        Self { tree }
    }
}

pub struct ProviderEventHandler<M>
where
    M: std::fmt::Debug + 'static,
{
    tree: Arc<Mutex<Option<IndexedTree<M>>>>,
    _phantom: PhantomData<M>,
}

impl<M> ProviderEventHandler<M>
where
    M: std::fmt::Debug + 'static,
{
    pub fn new(tree: Arc<Mutex<Option<IndexedTree<M>>>>) -> Self {
        Self {
            tree,
            _phantom: PhantomData,
        }
    }

    fn update_filedata(
        &self,
        node_id: NodeId,
        data: FileData,
    ) -> Result<iced::Task<M>, crate::Error> {
        let mut guard = self
            .tree
            .try_lock_for(Duration::from_secs(2))
            .ok_or(Error::Sync(SyncError::Deadlock(
                "Trying to lock tree on FileLoaded event".into(),
            )))?;

        let tree = guard.as_mut().unwrap();

        let node = tree
            .get_node_mut(&node_id)
            .ok_or(Error::NodeNotFound(node_id.clone()))?;

        node.with_data_mut(|data_node| {
            let res = match data_node.content_mut() {
                crate::node::Content::Value(value) => match data {
                    crate::data::FileData::Svg(handle) => match value.inner_mut() {
                        ValueKind::Dynamic { data, provider: _ } => {
                            data.replace(Arc::new(DataType::Svg(handle)));
                            Ok(())
                        }
                        _ => Err(Error::Unhandled(
                            "Expecting Value::Data in Svg handler".into(),
                        )),
                    },
                    crate::data::FileData::Image(handle) => match value.inner_mut() {
                        ValueKind::Dynamic { data, provider: _ } => {
                            data.replace(Arc::new(DataType::Image(handle)));
                            Ok(())
                        }
                        _ => Err(Error::Unhandled(
                            "Expecting Value::Data in Svg handler".into(),
                        )),
                    },
                    crate::data::FileData::Markdown(items) => match value.inner_mut() {
                        ValueKind::Dynamic { data, provider: _ } => {
                            data.replace(Arc::new(DataType::Markdown(MarkdownItems::new(items))));
                            Ok(())
                        }
                        _ => Err(Error::Unhandled(
                            "Expecting Value::Data in Svg handler".into(),
                        )),
                    },
                    crate::data::FileData::Text(text) => match value.inner_mut() {
                        ValueKind::Dynamic { data, provider: _ } => {
                            data.replace(Arc::new(DataType::Text(text)));
                            Ok(())
                        }
                        _ => Err(Error::Unhandled(
                            "Expecting Value::Data in Svg handler".into(),
                        )),
                    },
                },
                _ => Err(Error::Unhandled(
                    "Unknown Value node in FileLoaded event".into(),
                )),
            };

            if res.is_ok() {
                data_node.set_dirty(true);
            }

            res
        })?;

        Ok(Task::none())
    }
}

impl<M> EventHandler<M> for ProviderEventHandler<M>
where
    M: Clone + std::fmt::Debug + 'static,
{
    type Event = ProviderEvent;
    type State = Arc<Mutex<ProviderState<M>>>;

    fn handle(
        &self,
        event: Self::Event,
        _state: Self::State,
    ) -> Result<iced::Task<M>, crate::Error> {
        debug!("{event:?}");
        let task = match event {
            ProviderEvent::Initialized => Task::none(),
            ProviderEvent::Updated => todo!(),
            ProviderEvent::FileLoaded { node_id, data } => self.update_filedata(node_id, data)?,
            ProviderEvent::UrlLoaded {
                node_id,
                url: _,
                data,
            } => self.update_filedata(node_id, data)?,
            ProviderEvent::Error(_) => todo!(),
        };

        Ok(task)
    }
}
