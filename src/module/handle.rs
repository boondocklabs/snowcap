use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use iced::advanced::graphics::futures::{MaybeSend, MaybeSync};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

use crate::{NodeRef, SyncError};

use super::{data::ModuleData, event::ModuleEvent, DynModule, Module, ModuleHandleId};

/// Raw module handle, wrapping a dyn Module in Arc and tokio RwLock.
/// This is Send+Sync and can cross await boundaries.
#[derive(Debug)]
pub struct ModuleHandleRaw<E, D>
where
    Self: MaybeSend + MaybeSync,
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
    D: ModuleData + MaybeSend + MaybeSync + 'static,
{
    module: Arc<RwLock<DynModule<E, D>>>,
}

impl<E, D> ModuleHandleRaw<E, D>
where
    Self: MaybeSend + MaybeSync,
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
    D: ModuleData + MaybeSend + MaybeSync + 'static,
{
    /// Get a reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_async(&self) -> ModuleRef<E, D> {
        let guard = self.clone().module.read_owned().await;

        ModuleRef { guard }
    }

    /// Get a mutable reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_mut_async(&self) -> ModuleRefMut<E, D> {
        let guard = self.clone().module.write_owned().await;

        ModuleRefMut { guard }
    }

    /// Get a reference to the underlying dynamic [`Module`]
    pub fn try_module(&self) -> Result<ModuleRef<E, D>, SyncError> {
        let guard = self
            .clone()
            .module
            .try_read_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRef { guard })
    }

    /// Get a mutable reference to the underlying module
    pub fn try_module_mut(&self) -> Result<ModuleRefMut<E, D>, SyncError> {
        let guard = self
            .clone()
            .module
            .try_write_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRefMut { guard })
    }
}

impl<E, D> Clone for ModuleHandleRaw<E, D>
where
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
    D: ModuleData + MaybeSend + MaybeSync + 'static,
{
    fn clone(&self) -> Self {
        ModuleHandleRaw {
            module: self.module.clone(),
        }
    }
}

/// Handle to a [`Module`]. Each instantiated module is wrapped in a ModuleHandle
/// which wraps a dyn Module in a tokio Rwlock, wrapped in an Arc.
///
/// This handle can be cloned, and references to the inner module can be obtained
/// using the instance methods [`ModuleHandle::module()`] or [`ModuleHandle::module_mut()`]
///
/// To cross await boundaries, the handle can be converted to [`ModuleHandleRaw`] with [`ModuleHandle::into_raw()`]
#[derive(Debug)]
pub struct ModuleHandle<E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    /// Instance ID of this module
    id: ModuleHandleId,

    /// Name of the Module
    name: String,

    /// Reference to the root subtree node within the main Snowcap tree manageed by this module
    subtree: Option<NodeRef<E>>,

    /// Dynamic dispatch module handle wrapped in Arc and tokio RwLock
    /// This is in ModuleHandleRaw which is Send+Sync, to allow crossing
    /// await boundaries
    raw: ModuleHandleRaw<E, D>,
}

impl<E, D> ModuleHandle<E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    pub fn id(&self) -> ModuleHandleId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn subtree(&self) -> Option<&NodeRef<E>> {
        self.subtree.as_ref()
    }

    pub fn into_raw(self) -> ModuleHandleRaw<E, D> {
        self.raw
    }
}

impl<E, D> DerefMut for ModuleHandle<E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl<E, D> Deref for ModuleHandle<E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    type Target = ModuleHandleRaw<E, D>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

/// Clone a ModuleHandle, which clones the inner Arc/RwLock
/// for shared access to the underlying module.
impl<E, D> Clone for ModuleHandle<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            raw: self.raw.clone(),
            subtree: None,
        }
    }
}

impl<E, D> ModuleHandle<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    pub fn new(
        name: String,
        id: ModuleHandleId,
        module: impl Module<Event = E, Data = D> + 'static,
    ) -> Self {
        let m: DynModule<E, D> = Box::new(module);
        let module = Arc::new(RwLock::new(m));

        Self {
            name,
            id,
            raw: ModuleHandleRaw { module },
            subtree: None,
        }
    }
}

/// Immutable reference to a dynamic [`Module`]. Holds an ArcRwLockReadGuard to a dyn Module
pub struct ModuleRef<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    guard: OwnedRwLockReadGuard<DynModule<E, D>>,
}

/// Deref to dynamic [`Module`] impl from a ModuleRef through the held RwLock read guard
impl<E, D> Deref for ModuleRef<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    type Target = DynModule<E, D>;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

/// Mutable reference to a dynamic [`Module`]. Holds an ArcRwLockWriteGuard to a dyn Module
pub struct ModuleRefMut<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    guard: OwnedRwLockWriteGuard<DynModule<E, D>>,
}

impl<E, D> Deref for ModuleRefMut<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    type Target = DynModule<E, D>;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<E, D> DerefMut for ModuleRefMut<E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}
