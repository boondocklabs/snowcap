use std::sync::Arc;

use iced::advanced::graphics::futures::{MaybeSend, MaybeSync};

use super::message::ModuleMessageKind;

// Module Event trait
pub trait ModuleEvent: Send + Sync + std::fmt::Debug {}

/// For any T which implements [`ModuleEvent`], implement From<T> on ModuleMessageKind
/// which wraps the event in an Arc and creates a ModuleMessageKind::Event variant
impl<T: ModuleEvent + MaybeSend + MaybeSync + 'static> From<T> for ModuleMessageKind {
    fn from(value: T) -> Self {
        ModuleMessageKind::Event(Arc::new(Box::new(value)))
    }
}
