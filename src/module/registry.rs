use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc, LazyLock, Mutex},
};

use tracing::{debug, debug_span};

use colored::Colorize as _;

use super::{dispatch::ModuleDispatch, error::ModuleError, internal::ModuleInit, Module};

static MODULE_HANDLE_ID: LazyLock<Arc<AtomicU64>> = LazyLock::new(|| Arc::new(AtomicU64::new(0)));

static MODULE_REGISTRY: LazyLock<Mutex<HashMap<String, ModuleDescriptor>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Type alias for a boxed dyn closure which calls [`ModuleInit::new()`] and returns
/// a type erased [`ModuleDispatch`] instance to call into the module
pub type DynModuleNew = Box<dyn Fn() -> ModuleDispatch + Send + Sync + 'static>;

pub struct ModuleDescriptor {
    pub name: String,
    pub new: DynModuleNew,
}

pub struct ModuleRegistry;

impl ModuleRegistry {
    /// Register a [`ModuleDescriptor`] with the global module registry
    pub fn register_descriptor(descriptor: ModuleDescriptor) {
        if let Ok(mut registry) = MODULE_REGISTRY.try_lock() {
            registry.insert(descriptor.name.clone(), descriptor);
        } else {
            panic!("Failed to get module registry");
        }
    }

    /// Register a module with the global registry under the supplied name
    pub fn register<T: ModuleInit + Module>(name: &str) {
        debug_span!("module-register").in_scope(|| {
            debug!(
                "Registering module '{}' [{}, {}]",
                name.bright_green(),
                T::type_name().bright_blue(),
                T::event_name().bright_blue(),
            );

            let name = String::from(name);

            // Create a closure to proxy to the non-object safe ModuleInit::new(),
            // returning a type erased dispatcher
            let idgen = MODULE_HANDLE_ID.clone();
            let module_name = name.clone();
            let module_new = Box::new(move || {
                let id = idgen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // Call ModuleInit::new() to get a ModuleHandle
                let handle = T::new(module_name.clone(), id);

                // Create a dispatcher for type erasure
                let dispatcher = ModuleDispatch::new(handle);

                // Return the dispatcher
                dispatcher
            });

            let descriptor = ModuleDescriptor {
                name,
                new: module_new,
            };

            ModuleRegistry::register_descriptor(descriptor);
        })
    }

    pub fn get<R, F>(name: &str, f: F) -> Result<R, ModuleError>
    where
        F: FnOnce(&ModuleDescriptor) -> Result<R, ModuleError>,
    {
        if let Ok(registry) = MODULE_REGISTRY.try_lock() {
            if let Some(descriptor) = registry.get(name) {
                f(descriptor)
            } else {
                Err(ModuleError::ModuleNotFound(name.to_string()))
            }
        } else {
            panic!("Failed to acquire module registry lock");
        }
    }
}

/*
#[derive(EnumString, strum::Display)]
#[strum(ascii_case_insensitive)]
pub enum ModuleKind {
    Timing,
    Http,
}

impl ModuleManager {
    /// Create an internal module from the registry
    pub fn internal(&mut self, kind: ModuleKind) -> HandleId {
        match kind {
            ModuleKind::Timing => self.create::<super::timing::TimingModule>(),
            ModuleKind::Http => self.create::<super::http::HttpModule>(),
        }
    }

    pub fn from_string(name: &str) -> Result<ModuleKind, ModuleError> {
        match ModuleKind::from_str(name) {
            Ok(kind) => Ok(kind),
            Err(_e) => Err(ModuleError::Unknown(name.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module::manager::ModuleManager;

    #[test]
    fn from_string() {
        let res = ModuleManager::from_string("notexist".into());
        assert!(res.is_err());

        let res = ModuleManager::from_string("timing".into());
        assert!(res.is_ok());

        let res = ModuleManager::from_string("Timing".into());
        assert!(res.is_ok());

        let res = ModuleManager::from_string("TIMING".into());
        assert!(res.is_ok());
    }
}
*/
