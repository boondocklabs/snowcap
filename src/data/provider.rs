use std::sync::Arc;

use crate::{connector::Inlet, message::Event, parser::NodeId};
use iced::{
    advanced::graphics::futures::{MaybeSend, MaybeSync},
    Task,
};
use parking_lot::Mutex;
use url::Url;

use super::FileData;

pub trait Provider: std::fmt::Debug + MaybeSend + MaybeSync {
    fn init_task(&mut self, this: Arc<Mutex<dyn Provider>>, node_id: NodeId) -> Task<Event>;
    fn set_node_id(&mut self, node_id: NodeId);
    fn set_event_inlet(&self, inlet: Inlet<Event>);
    fn update_task(&self) -> Task<Event>;
}

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    Updated,
    FileLoaded {
        node_id: NodeId,
        data: FileData,
    },
    UrlLoaded {
        node_id: NodeId,
        url: Url,
        data: FileData,
    },
    Error(String),
}
