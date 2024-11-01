use std::sync::Arc;

use crate::{connector::Inlet, message::Event, ConversionError};
use arbutus::NodeId;
use iced::{
    advanced::graphics::futures::{MaybeSend, MaybeSync},
    Task,
};
use parking_lot::Mutex;
use url::Url;

use super::FileData;

pub(crate) type DynProvider = dyn Provider<H = crate::SnowHasher>;

pub trait Provider: std::fmt::Debug + std::fmt::Display + MaybeSend + MaybeSync {
    type H: std::hash::Hasher;

    fn init_task(&mut self, this: Arc<Mutex<DynProvider>>, node_id: NodeId) -> Task<Event>;
    fn set_node_id(&mut self, node_id: NodeId);
    fn set_event_inlet(&self, inlet: Inlet<Event>);
    fn update_task(&mut self) -> Task<Event>;
    fn hash_source(&self, hasher: &mut dyn std::hash::Hasher);
}

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    Initialized,
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

impl TryFrom<Event> for ProviderEvent {
    type Error = crate::ConversionError;

    fn try_from(event: Event) -> Result<Self, crate::ConversionError> {
        if let Event::Provider(provider_event) = event {
            Ok(provider_event)
        } else {
            Err(ConversionError::InvalidType(
                "expecting Event::Provider".into(),
            ))
        }
    }
}
