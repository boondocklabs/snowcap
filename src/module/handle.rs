use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use iced::advanced::graphics::futures::{MaybeSend, MaybeSync};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

use crate::{NodeRef, SyncError};

use super::{event::ModuleEvent, DynModule, HandleId, Module};

/// Raw module handle, wrapping a dyn Module in Arc and tokio RwLock.
/// This is Send+Sync and can cross await boundaries.
#[derive(Debug)]
pub struct ModuleHandleRaw<E>
where
    Self: MaybeSend + MaybeSync,
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
{
    module: Arc<RwLock<DynModule<E>>>,
}

impl<E> ModuleHandleRaw<E>
where
    Self: MaybeSend + MaybeSync,
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
{
    /// Get a reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_async(&self) -> ModuleRef<E> {
        let guard = self.clone().module.read_owned().await;

        ModuleRef { guard }
    }

    /// Get a mutable reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_mut_async(&self) -> ModuleRefMut<E> {
        let guard = self.clone().module.write_owned().await;

        ModuleRefMut { guard }
    }

    /// Get a reference to the underlying dynamic [`Module`]
    pub fn try_module(&self) -> Result<ModuleRef<E>, SyncError> {
        let guard = self
            .clone()
            .module
            .try_read_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRef { guard })
    }

    /// Get a mutable reference to the underlying module
    pub fn try_module_mut(&self) -> Result<ModuleRefMut<E>, SyncError> {
        let guard = self
            .clone()
            .module
            .try_write_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRefMut { guard })
    }
}

impl<E> Clone for ModuleHandleRaw<E>
where
    E: ModuleEvent + MaybeSend + MaybeSync + 'static,
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
pub struct ModuleHandle<E>
where
    E: ModuleEvent + MaybeSend + 'static,
{
    /// Instance ID of this module
    id: HandleId,

    /// Reference to the root subtree node within the main Snowcap tree manageed by this module
    subtree: Option<NodeRef<E>>,

    /// Dynamic dispatch module handle wrapped in Arc and tokio RwLock
    /// This is in ModuleHandleRaw which is Send+Sync, to allow crossing
    /// await boundaries
    raw: ModuleHandleRaw<E>,
}

impl<E> ModuleHandle<E>
where
    E: ModuleEvent + MaybeSend + 'static,
{
    pub fn id(&self) -> HandleId {
        self.id
    }

    pub fn subtree(&self) -> Option<&NodeRef<E>> {
        self.subtree.as_ref()
    }

    pub fn into_raw(self) -> ModuleHandleRaw<E> {
        self.raw
    }
}

impl<E> DerefMut for ModuleHandle<E>
where
    E: ModuleEvent + MaybeSend + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl<E> Deref for ModuleHandle<E>
where
    E: ModuleEvent + MaybeSend + 'static,
{
    type Target = ModuleHandleRaw<E>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

/// Clone a ModuleHandle, which clones the inner Arc/RwLock
/// for shared access to the underlying module.
impl<E> Clone for ModuleHandle<E>
where
    E: ModuleEvent + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            raw: self.raw.clone(),
            subtree: None,
        }
    }
}

impl<E> ModuleHandle<E>
where
    E: ModuleEvent + 'static,
{
    pub fn new(id: HandleId, module: impl Module<Event = E> + 'static) -> Self {
        let m: DynModule<E> = Box::new(module);
        let module = Arc::new(RwLock::new(m));

        Self {
            id,
            raw: ModuleHandleRaw { module },
            subtree: None,
        }
    }
}

/// Immutable reference to a dynamic [`Module`]. Holds an ArcRwLockReadGuard to a dyn Module
pub struct ModuleRef<E>
where
    E: ModuleEvent + 'static,
{
    guard: OwnedRwLockReadGuard<DynModule<E>>,
}

/// Deref to dynamic [`Module`] impl from a ModuleRef through the held RwLock read guard
impl<E> Deref for ModuleRef<E>
where
    E: ModuleEvent + 'static,
{
    type Target = DynModule<E>;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

/// Mutable reference to a dynamic [`Module`]. Holds an ArcRwLockWriteGuard to a dyn Module
pub struct ModuleRefMut<E>
where
    E: ModuleEvent + 'static,
{
    guard: OwnedRwLockWriteGuard<DynModule<E>>,
}

impl<E> Deref for ModuleRefMut<E>
where
    E: ModuleEvent + 'static,
{
    type Target = DynModule<E>;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<E> DerefMut for ModuleRefMut<E>
where
    E: ModuleEvent + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}
