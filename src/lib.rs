//! # Snowcap GUI Markup Engine
//!
//! <div class="warning">
//! Warning: Snowcap is under active development, and is very unstable.
//! Much of the planned functionality is unimplemented, and things will break frequently.
//! </div>
//!
//! Snowcap is a GUI Markup Engine based on [`iced`], it provides a [`pest`] based grammar to
//! process markup sources, and parse them into a tree using [`arbutus`](https://github.com/boondocklabs/arbutus).
//!
//! A [`snowcap-viewer`] application is available which serves as a good example of a top level application built on [`snowcap`].
//!
//! ## Hot Reloading
//! Hot reloading is a key goal of [`snowcap`]. Markup files loaded with [`Snowcap::load_file()`] are monitored for changes using [`notify`], and will
//! automatically be reloaded on change.
//!
//! ## Tree Diffing
//! Tree diffing using Xxh64 hashes is implemented in [`arbutus`] and used to determine changes between the trees, and only affected nodes are
//! replaced from the new tree into the live tree. Dirty paths are then marked and rebuilt in the [`Snowcap::update()`] phase.
//!
//! ## Widget Caching
//!
//! Snowcap caches widgets in-tree, and a root [`iced::Element`] is created from the root widget by reference on each [`Snowcap::view()`] phase.
//!
//! Each underlying iced Widget is wrapped in a [`DynamicWidget`] in an `Arc<RwLock>` and owned by a [`SnowcapNode`] (the tree node container).
//! Each [`DynamicWidget`] instance can issue at most one active [`dynamic_widget::WidgetRef`], which contains an owned [`parking_lot::ArcRwLockWriteGuard`]
//! ensuring exclusive mutable access to the widget by reference. This reference can then be converted to an [`iced::Element`], and iced then Deref's through
//! the guard back into Snowcap owned widgets in the tree.
//!
//! When the tree needs to be updated after a diff, the tree is iterated in reverse starting from the leaf nodes, and
//! any dirty [`DynamicWidget`] instances along affected paths are are dropped, inherently dropping the held guards
//! along the path. Node references where widgets are dropped are collected into a queue during this iteration pass, and new
//! widgets are built from the queue (starting with leaves to build children first), and replaced in each [`SnowcapNode`].
//!
//! ## Dynamic Modules
//!
//! There is a module framework in [`module`] which allows for creation of dynamic functionality that can be referenced in the snowcap markup.
//! Modules can be defined in the markup as widget contents to provide their data from the network or a file, and they can subscribe to topics
//! and publish messages.
//!
//! Arguments can be specified for modules in the grammar using `{key: value (, key:value)+}` and are
//! passed to [`crate::module::Module::init()`] as [`crate::module::argument::ModuleArguments`].
//!
//! ### Internal Modules
//!
//! | Module              | Description                  | Example Grammar      |
//! |---------------------|-----------------------------------|----------------------|
//! | [`module::file`]    | Loading files from the filesystem | ```image(file!{path:"pic.png"}) // Get the contents of a PNG file for an image widget ```                    |
//! | [`module::http`]    | Making HTTP Network Requests      | ```text(http!{method:"get", url:"http://icanhazip.com"}) // Get the contents of a URL into a text widget```  |
//! | [`module::timing`]  | Timing related functionality      | ```timing!{periodic:"1s"}  // Periodic timer triggering every second```                                      |
//!
//!
//! ### Custom Modules
//! Custom modules can be defined by implementing [`module::Module`] on your own struct, and registering it with the engine using [`Snowcap::modules()`]
//! to get the [`ModuleManager`], and calling [`ModuleManager::register()`].
//!
//!
//! ```ignore
//! // Register a module with a Snowcap Engine
//! snowcap.modules().register<MyModule>("custom-module");
//! ```
//!
//! The sealed traits [`module::internal::ModuleInit`] and [`module::internal::ModuleInternal`] get automatically blanket implemented on
//! any type implementing [`module::Module`] to handle instantation and dynamic message dispatching.
//!
//! All that's generally required is implementing [`module::Module::init()`], and one or more of
//!
//! * [`module::Module::on_event()`] to handle internal messages defined by [`module::Module::Event`] associated type
//! * [`module::Module::on_subscription()`] to receive messages published to topics by other modules or the core engine
//!
//! In addition, a message type must be defined which implements [`module::event::ModuleEvent`] and set as the associated type [`module::Module::Event`].
//!
//! ## Grammar Definitions
//! The grammar for the markup format is defined in [`pest`] parser expression grammar (PEG).
//!
//! The full parser is split up between different PEG definitions
//!
//! #### Grammar Files
//! | Pest PEG                                |  Description      | Parser Implementation |
//! |-----------------------------------------|-------------------|-----------------------|
//! | [`src/snowcap.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/snowcap.pest)  | Top level grammar | [`parser::SnowcapParser`]
//! | [`src/parser/attribute.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/attribute.pest)  | Widget Attribute grammar | [`parser::attribute::AttributeParser`]
//! | [`src/parser/color.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/color.pest)  | Color grammar | [`parser::color::ColorParser`]
//! | [`src/parser/gradient.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/gradient.pest)  | Gradient grammar | [`parser::gradient::GradientParser`]
//! | [`src/parser/module.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/module.pest)  | Dynamic module grammar | [`parser::module::ModuleParser`]
//! | [`src/parser/value.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/value.pest)  | Value grammar | [`parser::value::ValueParser`]
//!
//! [`snowcap-viewer`]: https://github.com/boondocklabs/snowcap-viewer
//! [`snowcap`]: https://github.com/boondocklabs/snowcap
//! [`arbutus`]: https://github.com/boondocklabs/arbutus
//! [`iced`]: https://iced.rs
//! [`pest`]: https://pest.rs
//! [`notify`]: https://docs.rs/notify/latest/notify/

mod attribute;
//mod connector;
mod conversion;
mod data;
mod dynamic_widget;
mod error;
//mod event;
mod cache;
pub mod message;
pub mod module;
mod node;
mod parser;
//mod router;
mod util;
mod watcher;

pub use message::module::*;

use arbutus::TreeDiff;
use arbutus::TreeNode as _;
use arbutus::TreeNodeRef as _;
use dynamic_widget::DynamicWidget;

// Re-export iced
pub use iced;
use iced::Task;

use cache::WidgetCache;
use message::Command;
use module::manager::ModuleManager;
use module::ModuleHandleId;
use node::SnowcapNode;
use parking_lot::Mutex;
use salish::endpoint::Endpoint;
use salish::router::MessageRouter;
use watcher::FileWatcher;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

pub use conversion::theme::SnowcapTheme;
pub use error::*;
pub use salish::Message;

pub use parser::SnowcapParser;
pub use parser::Value;

use tracing::error;
use tracing::info;

//type Node<Data, Id> = arbutus::node::rc::Node<Data, Id>;
//type NodeRef<M> = arbutus::noderef::rc::NodeRef<Node<SnowcapNode<M>, arbutus::NodeId>>;

type Node<Data, Id> = arbutus::node::arc::Node<Data, Id>;
type NodeRef = arbutus::noderef::arc::NodeRef<Node<SnowcapNode, arbutus::NodeId>>;

type Tree = arbutus::Tree<NodeRef>;
type IndexedTree = arbutus::IndexedTree<NodeRef>;
type NodeId = arbutus::NodeId;

#[derive(Debug, Clone, Copy, Hash)]
pub enum Source {
    Module(ModuleHandleId),
}

/// Top level Snowcap Engine which manages loading and parsing grammar into an [`Arbutus`](https://github.com/boondocklabs/arbutus) tree.
/// Provides the update() and view()
pub struct Snowcap {
    #[cfg(not(target_arch = "wasm32"))]
    filename: Option<PathBuf>,
    tree: Arc<Mutex<Option<IndexedTree>>>,
    modules: Rc<RefCell<ModuleManager>>,
    watcher: Option<FileWatcher>,

    router: MessageRouter<'static, Task<salish::message::Message>, Source>,

    cache: Rc<RefCell<WidgetCache>>,

    _command_endpoint: Endpoint<'static, Command, Task<Message>, Source>,
}

impl Snowcap {
    /// Create a new Snowcap Engine instance
    pub fn new() -> Result<Self, Error> {
        let router = MessageRouter::<Task<Message>, Source>::new();

        let tree = Arc::new(Mutex::new(None));
        let modules = Rc::new(RefCell::new(ModuleManager::new(router.clone())));

        let command_endpoint = router
            .create_endpoint::<Command>()
            .message(|source, command| match command {
                Command::Shutdown => {
                    println!("Shutdown command received from {source:?}");
                    iced::exit()
                }
                Command::Reload => todo!(),
            });

        let snow = Self {
            tree,
            #[cfg(not(target_arch = "wasm32"))]
            filename: None,
            modules,
            watcher: None,
            router,
            _command_endpoint: command_endpoint,
            cache: Rc::new(RefCell::new(WidgetCache::default())),
        };

        Ok(snow)
    }

    /// Engine initialization, called by [`iced::Application`].
    /// Traverses the tree to build widgets, and gets an init [`iced::Task`]
    /// from each instantiated [`module`] in the tree.
    pub fn init(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();

        let (watcher, watcher_task) = FileWatcher::new();

        self.watcher = Some(watcher);

        tasks.push(watcher_task);

        if let Some(filename) = &self.filename {
            self.watcher.as_mut().unwrap().watch(filename).unwrap();
        }

        // Run the initial tree update, and get any tasks (Provider init tasks)
        let tree_task = if let Some(tree) = &*self.tree.lock() {
            profiling::scope!("build-widgets");
            info!("{}", tree.root());
            let mut cache = self.cache.borrow_mut();
            match cache.update_tree(tree, &mut self.modules_mut()) {
                Ok(task) => task,
                Err(e) => {
                    error!("Failed to build widgets: {}", e);
                    Task::none()
                }
            }
        } else {
            Task::none()
        };

        tasks.push(tree_task);

        info!("Starting init tasks");

        Task::batch(tasks)
    }

    /// Get a reference to the [`ModuleManager`] for registering and instantiating modules at runtime
    pub fn modules(&self) -> std::cell::Ref<'_, ModuleManager> {
        self.modules.borrow()
    }

    /// Get a mutable reference to the [`ModuleManager`] for registering and instantiating modules at runtime
    pub fn modules_mut(&self) -> std::cell::RefMut<'_, ModuleManager> {
        self.modules.borrow_mut()
    }

    /// Get a reference to the [`MessageRouter`]
    pub fn router(&mut self) -> &mut MessageRouter<'static, Task<Message>, Source> {
        &mut self.router
    }

    /// Load a markup file and set the active [`arbutus`] tree.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_file(&mut self, filename: String) -> Result<(), Error> {
        use colored::Colorize;

        let filename = &PathBuf::from(&filename);
        let tree = SnowcapParser::<Message>::parse_file(&filename)?;

        let tree = IndexedTree::from_tree(tree);

        println!(
            "\n{}\n{}",
            "Snowcap file loaded into tree:".magenta(),
            tree.root()
        );

        self.filename = Some(filename.clone());

        self.set_tree(tree)?;

        Ok(())
    }

    /// Load markup from memory. If a tree is currently loaded, the new tree is diffed
    /// and changes are patched into the existing tree.
    pub fn load_memory(&mut self, data: &str) -> Result<(), Error> {
        let tree = SnowcapParser::<Message>::parse_memory(data)?;

        if let Some(current) = &mut *self.tree.lock() {
            // We already have a tree loaded. Diff the trees
            let mut diff = TreeDiff::new(current.root().clone(), tree.root().clone());
            let patch = diff.diff();

            info!("Patching existing tree {patch:#?}");
            patch.patch_tree(current);

            current.reindex();

            return Ok(());
        }

        self.set_tree(IndexedTree::from_tree(tree))?;

        Ok(())
    }

    fn set_tree(&mut self, tree: IndexedTree) -> Result<(), Error> {
        *self.tree.lock() = Some(tree);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn reload_file(&mut self) -> Result<(), Error> {
        use arbutus::TreeDiff;
        use colored::Colorize;

        let filename = self.filename.clone().ok_or(Error::MissingAttribute(
            "No snowcap grammar filename in self".to_string(),
        ))?;

        // Parse the new file into an IndexedTree
        let mut new_tree = IndexedTree::from_tree(SnowcapParser::<Message>::parse_file(&filename)?);

        let _listener = new_tree
            .on_event(|event| {
                println!("NEW TREE EVENT {event:?}");
            })
            .ok();

        println!("{}", "Parsed New Tree".bright_magenta());
        println!("{}", new_tree.root());

        if let Some(tree) = &mut (*self.tree.lock()) {
            // Register an event handler on the tree. It will automatically be deregistered when it goes out of scope.
            // This handler listens for tree modification events, and marks the nodes as dirty in the snowcap node data,
            // so the affected widgets will be rebuilt on the next update pass.
            let _listener = tree
                .on_event(|event| {
                    match event {
                        arbutus::TreeEvent::NodeRemoved { node } => {
                            if let Some(parent) = node.clone().node_mut().parent_mut() {
                                parent.node_mut().data_mut().set_state(node::State::Dirty)
                            }
                        }
                        arbutus::TreeEvent::NodeReplaced { node } => node
                            .clone()
                            .node_mut()
                            .data_mut()
                            .set_state(node::State::New),
                        arbutus::TreeEvent::SubtreeInserted { node } => {
                            // Invalidate the whole subtree
                            for mut n in node {
                                n.node_mut().data_mut().set_state(node::State::New)
                            }
                        }
                        arbutus::TreeEvent::ChildRemoved { parent, .. } => parent
                            .clone()
                            .node_mut()
                            .data_mut()
                            .set_state(node::State::Dirty),
                        arbutus::TreeEvent::ChildrenRemoved { parent, .. } => parent
                            .clone()
                            .node_mut()
                            .data_mut()
                            .set_state(node::State::Dirty),
                        arbutus::TreeEvent::ChildrenAdded { parent, children } => {
                            for child in children {
                                child
                                    .clone()
                                    .node_mut()
                                    .data_mut()
                                    .set_state(node::State::New)
                            }
                            parent
                                .clone()
                                .node_mut()
                                .data_mut()
                                .set_state(node::State::Dirty)
                        }
                        arbutus::TreeEvent::ChildReplaced { parent, index }
                        | arbutus::TreeEvent::ChildInserted { parent, index } => {
                            // Invalidate the child
                            let mut parent = parent.clone();
                            let mut node = parent.node_mut();
                            let child = node.children_mut().unwrap().get_mut(*index).unwrap();

                            child.node_mut().data_mut().set_state(node::State::New);
                        }
                    };
                })
                .unwrap();

            let mut diff = TreeDiff::new(tree.root().clone(), new_tree.root().clone());
            let patch = diff.diff();
            patch.patch_tree(tree);

            tree.reindex();
        }

        Ok(())
    }

    /*
    fn handle_widget_message(
        &mut self,
        node_id: &NodeId,
        message: &mut WidgetMessage,
    ) -> Task<Message<AppMessage>> {
        info_span!("widget-message").in_scope(|| {

            // Set the widget as dirty in the tree
            if let Some(node) = self.tree.lock().as_mut().unwrap().get_node_mut(node_id) {
                node.node_mut().data_mut().set_dirty(true);
            }

            match message {
                WidgetMessage::Markdown(url) => {
                    info!("Markdown URL click {url}");
                    Task::none()
                }
                WidgetMessage::ButtonPress(id) => {
                    info!("Button pressed node={id:?}");
                    Task::none()
                }
                WidgetMessage::Toggler {
                    element_id: id,
                    toggled,
                } => {
                    info!("Toggler node={id:?} toggled={toggled}");
                    Task::none()
                }
                WidgetMessage::PickListSelected {
                    element_id: id,
                    selected,
                } => {
                    info!("Picklist selected node={id:?} selected={selected}");
                    Task::none()
                }

                WidgetMessage::SliderChanged {
                    element_id: id,
                    value,
                } => {
                    info!("Slider changed node_id={node_id} element_id={id:?} value={value}");
                    Task::none()
                }

                WidgetMessage::SliderReleased { element_id, value } => {
                    info!(
                    "Slider released node_id={node_id} element_id={element_id:?} value={value}"
                    );

                    Task::none()
                }

                WidgetMessage::Scrolled { element_id, viewport } => {
                    info!("Scrolled node_id={node_id} element_id={element_id:?} viewport={viewport:?}");
                    Task::none()
                }
            }
        })
    }
    */

    #[profiling::function]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        /*
        let task = {
            profiling::scope!("message-dispatch");

            let message = std::mem::take(message);

            let task = match message {
                Message::Module(module_message) => {
                    // Handle data messages to update the tree
                    if let ModuleMessage::Data(data) = module_message.message() {
                        if let Some(node_id) =
                            self.modules.get_module_node(module_message.handle_id())
                        {
                            if let Some(tree) = &mut *self.tree.lock() {
                                if let Some(node) = tree.get_node_mut(&node_id) {
                                    node.node_mut().data_mut().set_module_data(data.clone());
                                }
                            }
                        }

                        Task::none()
                    } else {
                        self.modules
                            .handle_message(module_message)
                            .map(|m| Message::from(m))
                    }
                }
                Message::App(app_message) => {
                    debug!("AppMessage {app_message:?}");
                    //f(&app_message)
                    Task::none()
                }
                Message::Widget {
                    node_id,
                    mut message,
                } => self.handle_widget_message(&node_id, &mut message),
                Message::Event(event) => {
                    //let event = std::mem::take(event);
                    self.handle_event(event)
                }
                Message::Empty => {
                    warn!("Update called on Empty Message");
                    Task::none()
                }

                Message::Command(Command::Reload) => {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Err(e) = self.reload_file() {
                        error!("{}", e);
                    }
                    Task::none()
                }

                Message::Watcher(msg) => {
                    println!("WATCHER MESSAGE SNOWCAP HANDLER");

                    Task::none()
                }
            };
            task

            Task::none()
        };
        */

        // Pass the message to the router, and create a batch of returned tasks
        let router_task = if let Some(tasks) = self.router.handle_message(message) {
            Task::batch(tasks)
        } else {
            Task::none()
        };

        let tree_task = if let Some(tree) = &*self.tree.lock() {
            profiling::scope!("build-widgets");
            info!("{}", tree.root());
            let mut cache = self.cache.borrow_mut();
            match cache.update_tree(tree, &mut self.modules_mut()) {
                Ok(task) => task,
                Err(e) => {
                    error!("Failed to build widgets: {}", e);
                    Task::none()
                }
            }
        } else {
            Task::none()
        };

        //tree_task.chain(router_task)

        // Run the router tasks, followed by tree update tasks
        router_task.chain(tree_task)
    }

    #[profiling::function]
    pub fn view<'b>(&'b self) -> iced::Element<'b, Message> {
        info!("View");

        let root = if let Some(tree) = &*self.tree.lock() {
            let root_id = tree.root().node().id();

            if let Some(widget) = self.cache.borrow().get(root_id) {
                widget.into_element().unwrap()
            } else {
                iced::widget::Text::new("No root widget in tree").into()
            }
        } else {
            iced::widget::Text::new("No tree").into()
        };

        profiling::finish_frame!();
        root
    }
}
