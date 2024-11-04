use std::sync::Arc;

use super::message::ModuleMessage;

// Module Event trait
pub trait ModuleEvent: Send + Sync + std::fmt::Debug {}

/// For any T which implements [`ModuleEvent`], implement From<T> on [`ModuleMessage`]
/// which wraps the event in an Arc and creates a ModuleMessageKind::Event variant
impl<T: ModuleEvent + 'static> From<T> for ModuleMessage {
    fn from(value: T) -> Self {
        ModuleMessage::Event(Arc::new(Box::new(value)))
    }
}
