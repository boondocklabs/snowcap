use std::{marker::PhantomData, sync::Arc};

use arbutus::NodeRef as _;

use iced::Task;
use parking_lot::Mutex;
use tracing::{debug, error};

use crate::{
    data::{provider::ProviderEvent, DataType, MarkdownItems},
    Error, IndexedTree,
};

use super::EventHandler;

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
        match event {
            ProviderEvent::Updated => todo!(),
            ProviderEvent::FileLoaded { node_id, data } => {
                let mut guard = self.tree.lock();
                let tree = guard.as_mut().unwrap();

                let node = tree
                    .get_node_mut(&node_id)
                    .ok_or(Error::NodeNotFound(node_id.clone()))?;

                debug!("File loaded for node {node:#?}");

                node.with_data_mut(|mut data_node| match &mut data_node.data {
                    crate::node::SnowcapNodeData::Value(value) => match data {
                        crate::data::FileData::Svg(handle) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
                                data.replace(Arc::new(DataType::Svg(handle)));
                                Ok(())
                            }
                            _ => Err(Error::Unhandled(
                                "Expecting Value::Data in Svg handler".into(),
                            )),
                        },
                        crate::data::FileData::Image(handle) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
                                data.replace(Arc::new(DataType::Image(handle)));
                                Ok(())
                            }
                            _ => Err(Error::Unhandled(
                                "Expecting Value::Data in Svg handler".into(),
                            )),
                        },
                        crate::data::FileData::Markdown(items) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
                                data.replace(Arc::new(DataType::Markdown(MarkdownItems::new(
                                    items,
                                ))));
                                Ok(())
                            }
                            _ => Err(Error::Unhandled(
                                "Expecting Value::Data in Svg handler".into(),
                            )),
                        },
                        crate::data::FileData::Text(text) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
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
                })?;
            }
            ProviderEvent::UrlLoaded { node_id, url, data } => {
                let mut guard = self.tree.lock();
                let tree = guard.as_mut().unwrap();

                let node = tree
                    .get_node_mut(&node_id)
                    .ok_or(Error::NodeNotFound(node_id.clone()))?;
                debug!("URL '{url:?}' loaded for node {node:#?}");

                node.with_data_mut(|mut data_node| match &mut data_node.data {
                    crate::node::SnowcapNodeData::Value(value) => match data {
                        crate::data::FileData::Svg(_handle) => todo!(),
                        crate::data::FileData::Image(handle) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
                                data.replace(Arc::new(DataType::Image(handle)));
                                Ok(())
                            }
                            _ => Err(Error::Unhandled(
                                "Expecting Value::Data in Svg handler".into(),
                            )),
                        },
                        crate::data::FileData::Markdown(_vec) => todo!(),
                        crate::data::FileData::Text(text) => match value {
                            crate::Value::Dynamic { data, provider: _ } => {
                                data.replace(Arc::new(DataType::Text(text)));
                                Ok(())
                            }
                            _ => Err(Error::Unhandled(
                                "Expecting Value::Data in Svg handler".into(),
                            )),
                        },
                    },

                    _ => Err(Error::Unhandled(
                        "Unknown Value node in UrlLoaded event".into(),
                    )),
                })?;
            }
            ProviderEvent::Error(err) => error!("Provider Error Event: {err}"),
        }

        Ok(Task::none())
    }
}
