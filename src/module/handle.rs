//! Dynamic Module Handle

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use iced::{advanced::graphics::futures::MaybeSend, Task};
use salish::{router::MessageRouter, Message};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};
use tracing::{debug, instrument};

use crate::{Source, SyncError};

use super::{data::ModuleData, event::ModuleEvent, DynModule, Module, ModuleHandleId};

/// Handle to a [`Module`]. Each instantiated module is wrapped in a ModuleHandle
/// which wraps a dyn Module in a tokio Rwlock, wrapped in an Arc.
///
/// This handle can be cloned, and references to the inner module can be obtained
/// using the instance methods [`ModuleHandle::module()`] or [`ModuleHandle::module_mut()`]
///
/// To cross await boundaries, the handle can be converted to [`ModuleHandleRaw`] with [`ModuleHandle::into_raw()`]
#[derive(Debug)]
pub struct ModuleHandle<'a, E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    /// Instance ID of this module
    id: ModuleHandleId,

    /// Name of the Module
    name: String,

    router: Option<MessageRouter<'a, Task<Message>, Source>>,

    module: Arc<RwLock<DynModule<E, D>>>,
}

impl<'a, E, D> ModuleHandle<'a, E, D>
where
    E: ModuleEvent + MaybeSend + 'static,
    D: ModuleData + MaybeSend + 'static,
{
    /// Get the [`ModuleHandleId`] of this module instance
    pub fn id(&self) -> ModuleHandleId {
        self.id
    }

    /// Get the name of this module
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the [`Router`] of this module
    pub fn router(&self) -> Option<&MessageRouter<'a, Task<Message>, Source>> {
        self.router.as_ref()
    }

    /// Get a reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_async(&self) -> ModuleRef<E, D> {
        let guard = self.module.clone().read_owned().await;

        ModuleRef { guard }
    }

    /// Get a mutable reference to the underlying dynamic [`Module`], awaiting the tokio RwLock
    pub async fn module_mut_async(&self) -> ModuleRefMut<E, D> {
        let guard = self.module.clone().write_owned().await;

        ModuleRefMut { guard }
    }

    /// Get a reference to the underlying dynamic [`Module`]
    pub fn try_module(&self) -> Result<ModuleRef<E, D>, SyncError> {
        let guard = self
            .module
            .clone()
            .try_read_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRef { guard })
    }

    /// Get a mutable reference to the underlying module
    pub fn try_module_mut(&self) -> Result<ModuleRefMut<E, D>, SyncError> {
        let guard = self
            .module
            .clone()
            .try_write_owned()
            .map_err(|e| SyncError::from(e))?;

        Ok(ModuleRefMut { guard })
    }
}

/// Clone a ModuleHandle, which clones the inner Arc/RwLock
/// for shared access to the underlying module.
impl<'a, E, D> Clone for ModuleHandle<'a, E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            router: self.router.clone(),
            module: self.module.clone(),
        }
    }
}

impl<'a, E, D> ModuleHandle<'a, E, D>
where
    E: ModuleEvent + 'static,
    D: ModuleData + 'static,
{
    /// Create a new [`ModuleHandle`]
    pub fn new(
        name: String,
        id: ModuleHandleId,
        module: impl Module<Event = E, Data = D> + 'static,
    ) -> Self {
        let dyn_module: DynModule<E, D> = Box::new(module);
        let wrapped_module = Arc::new(RwLock::new(dyn_module));

        Self {
            name,
            id,
            module: wrapped_module,
            router: None,
        }
    }

    /// Add a router to this [`ModuleHandle`]
    #[instrument(name = "module")]
    pub fn with_router(mut self, router: MessageRouter<'a, Task<Message>, Source>) -> Self {
        debug!("Added Router to ModuleHandle");
        self.router = Some(router);
        self
    }
}

/// Immutable reference to a `dyn` [`Module`]. Holds an ArcRwLockReadGuard for the duration of the reference
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
