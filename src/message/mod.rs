//! Messages for internal snowcap, module, and application communications

pub mod module;
pub mod widget;

use std::{
    any::{Any, TypeId},
    hash::Hash,
    sync::Arc,
};

use strum::{EnumDiscriminants, EnumIter};
use widget::WidgetMessage;

use crate::{module::message::ModuleMessage, watcher::WatchMessage};

/// Message Kind
#[derive(Default, Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(MessageKind))]
pub enum MessageData {
    #[default]
    Empty,
    Shutdown,
    App(Arc<Box<dyn Any + Send + Sync>>),
    Widget(WidgetMessage),
    Command(Command),
    Watcher(WatchMessage),
    Module(ModuleMessage),
}

/// Get the [`TypeId`] of the inner variant of [`MessageData`]
/// This is used to match messages against registered handlers in [`MessageRouter`]
impl Into<TypeId> for &MessageData {
    fn into(self) -> TypeId {
        match self {
            MessageData::App(arc) => (*arc).type_id(),
            MessageData::Widget(widget_message) => (*widget_message).type_id(),
            MessageData::Command(command) => (*command).type_id(),
            MessageData::Watcher(watch_message) => (*watch_message).type_id(),
            MessageData::Module(module_message_container) => (*module_message_container).type_id(),

            // Message variants with no inner data use the self TypeId
            _ => self.type_id(),
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(CommandKind))]
pub enum Command {
    Shutdown,
    Reload,
}
