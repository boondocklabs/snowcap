//! Module data access trait
//!
//! Data objects created by modules are exposed into the core engine using the ModuleData trait.
//! When a widget wants to get content data from a module, it will call into the [`ModuleData`] impl

use super::error::ModuleError;

#[derive(Copy, Clone, Debug)]
pub enum ModuleDataKind {
    Unknown,
    Image,
    Svg,
    Text,
}

pub trait ModuleData: std::fmt::Debug + Send + Sync {
    fn kind(&self) -> ModuleDataKind;
    fn bytes(&self) -> Result<&Vec<u8>, ModuleError>;
}
