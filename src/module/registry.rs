use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc, LazyLock, Mutex},
};

use iced::Task;
use salish::{router::MessageRouter, Message};
use tracing::{debug, debug_span};

use colored::Colorize as _;

use crate::Source;

use super::{dispatch::ModuleDispatch, error::ModuleError, internal::ModuleInit, Module};

/// Module Handle ID generator. Each constructor closure in [`ModuleDescriptor`] keeps a clone of this
/// [`AtomicU64`] for allocating a new ID on each module instantiation.
static MODULE_HANDLE_ID: LazyLock<Arc<AtomicU64>> = LazyLock::new(|| Arc::new(AtomicU64::new(0)));

/// Global Module Registry. Each available module is registered into this registry at runtime
static MODULE_REGISTRY: LazyLock<Mutex<HashMap<String, ModuleDescriptor>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Type alias for a boxed dyn closure which calls [`ModuleInit::new()`] and returns
/// a type erased [`ModuleDispatch`] instance to call into the module
pub type DynModuleNew = Box<
    dyn for<'a> Fn(MessageRouter<'static, Task<Message>, Source>) -> ModuleDispatch
        + Send
        + Sync
        + 'static,
>;

/// Dynamic Module registration descriptor.
/// Each dynamic module which is available for instantiation has
/// an associated `ModuleDescriptor` that is inserted into the global
/// module registry.
///
/// Each descriptor contains a boxed closure [`DynModuleNew`] that calls [`ModuleInit::new()`]
/// of the specific [`Module`] implementation.
pub struct ModuleDescriptor {
    /// Name of this module
    pub name: String,

    /// Boxed closure proxying to [`ModuleInit::new()`] of this registered module
    pub new: DynModuleNew,
}

pub struct ModuleRegistry;

impl std::fmt::Display for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "--- Avaiable Modules:\n".bright_white())?;

        if let Ok(guard) = MODULE_REGISTRY.lock() {
            let mut keys: Vec<&String> = guard.keys().collect();
            keys.sort();
            keys.iter()
                .try_for_each(|k| write!(f, "{} {}\n", "|".bright_white(), k.cyan()))?;
        }

        write!(f, "{}", "---".bright_white())?;

        Ok(())
    }
}

impl ModuleRegistry {
    /// Register a [`ModuleDescriptor`] with the global module registry
    pub fn register_descriptor(descriptor: ModuleDescriptor) {
        if let Ok(mut registry) = MODULE_REGISTRY.lock() {
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

            // Clone the module handle ID generator to move into the module constructor closure
            let idgen = MODULE_HANDLE_ID.clone();

            // Clone the module name to move into the module constructor closure
            let module_name = name.clone();

            // Create a closure to proxy to the non-object safe ModuleInit::new(),
            // returning a type erased dispatcher
            let module_new: DynModuleNew = Box::new(move |router| {
                {
                    // Get a new module ID from the cloned AtomicU64 monotonic counter
                    let id = idgen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Call ModuleInit::new() to get a ModuleHandle
                    let handle = T::new(module_name.clone(), id).with_router(router);

                    // Create a dispatcher for type erasure
                    let dispatcher = ModuleDispatch::new(handle);

                    // Return the dispatcher
                    dispatcher
                }
            });

            // Create a [`ModuleDescriptor`] for this module registration
            let descriptor = ModuleDescriptor {
                name,
                new: module_new,
            };

            // Insert the descriptor into the global module registry
            ModuleRegistry::register_descriptor(descriptor);
        })
    }

    /// Get a module from the registry by name
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
