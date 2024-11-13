//use std::sync::Arc;
//use crate::message::module::ModuleMessageData;

// Module Event trait
pub trait ModuleEvent: Send + Sync + std::fmt::Debug {}

/*
/// For any T which implements [`ModuleEvent`], implement From<T> on [`ModuleMessage`]
/// which wraps the event in an Arc and creates a ModuleMessageKind::Event variant
impl<T: ModuleEvent + 'static> From<T> for ModuleMessageData {
    fn from(value: T) -> Self {
        ModuleMessageData::Event(Arc::new(Box::new(value)))
    }
}
*/
