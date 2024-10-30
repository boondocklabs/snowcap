mod attribute;
mod connector;
mod conversion;
mod data;
mod dynamic_widget;
mod error;
mod event;
mod message;
mod node;
mod parser;
mod tree_util;
mod util;
mod widget;

use arbutus::TreeDiff;
use arbutus::TreeNode as _;
use arbutus::TreeNodeRef as _;
use connector::{Endpoint, Inlet};
use data::provider::ProviderEvent;
use dynamic_widget::DynamicWidget;

#[cfg(not(target_arch = "wasm32"))]
use event::fsnotify::FsNotifyEventHandler;
#[cfg(not(target_arch = "wasm32"))]
use event::fsnotify::FsNotifyState;

use event::provider::ProviderEventHandler;
use event::provider::ProviderState;
use event::DynamicHandler;
use event::EventHandler;

// Re-export iced
pub use iced;
use iced::advanced::graphics::futures::MaybeSend;
use iced::futures;
use iced::futures::SinkExt;
use iced::Task;

use message::Command;
use message::Event;
use message::EventKind;
use message::MessageDiscriminants;
use message::WidgetMessage;
use node::Content;
use node::SnowcapNode;
use parking_lot::Mutex;
use parser::value::ValueKind;
use tracing::warn;
use tree_util::WidgetCache;
use xxhash_rust::xxh64::Xxh64;

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub use conversion::theme::SnowcapTheme;
pub use error::*;
pub use message::Message;

#[cfg(not(target_arch = "wasm32"))]
use notify::FsEventWatcher;
#[cfg(not(target_arch = "wasm32"))]
use notify::RecommendedWatcher;
#[cfg(not(target_arch = "wasm32"))]
use notify::Watcher;

pub use parser::SnowcapParser;
pub use parser::Value;

use tracing::debug;
use tracing::error;
use tracing::info;

/*
type Node<Data, Id> = arbutus::node::refcell::Node<Data, Id>;
type NodeRef<M> = arbutus::noderef::rc::NodeRef<Node<SnowcapNode<M>, arbutus::NodeId>>;
*/

type Node<Data, Id> = arbutus::node::simple::Node<Data, Id>;
type NodeRef<M> = arbutus::noderef::rc::NodeRef<Node<SnowcapNode<M>, arbutus::NodeId>>;

type Tree<M> = arbutus::Tree<NodeRef<M>>;
type IndexedTree<M> = arbutus::IndexedTree<NodeRef<M>>;
type NodeId = arbutus::NodeId;

type SnowHasher = Xxh64;

pub struct Snowcap<AppMessage>
where
    AppMessage: Clone + std::fmt::Debug + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    filename: Option<PathBuf>,

    tree: Arc<Mutex<Option<IndexedTree<Message<AppMessage>>>>>,

    #[cfg(not(target_arch = "wasm32"))]
    provider_watcher: FsEventWatcher,

    event_endpoint: Endpoint<Event>,

    event_handler: HashMap<EventKind, DynamicHandler<'static, Message<AppMessage>>>,

    #[cfg(not(target_arch = "wasm32"))]
    notify_state: Arc<Mutex<FsNotifyState<Message<AppMessage>>>>,
    provider_state: Arc<Mutex<ProviderState<Message<AppMessage>>>>,
}

impl<AppMessage> Snowcap<AppMessage>
where
    AppMessage: Clone + MaybeSend + std::fmt::Debug,
{
    pub fn new() -> Result<Self, Error> {
        let event_endpoint = Endpoint::new();

        info!("Event Endpoint ID: {}", event_endpoint.id());

        #[cfg(not(target_arch = "wasm32"))]
        // Initialize the filesystem watcher
        let provider_watcher = Self::init_watcher(event_endpoint.get_inlet());

        let tree = Arc::new(Mutex::new(None));

        #[cfg(not(target_arch = "wasm32"))]
        let notify_state = Arc::new(Mutex::new(FsNotifyState::new(tree.clone())));
        let provider_state = Arc::new(Mutex::new(ProviderState::new(tree.clone())));

        let mut snow = Self {
            tree,
            event_endpoint,
            event_handler: HashMap::new(),
            provider_state,
            #[cfg(not(target_arch = "wasm32"))]
            notify_state,
            #[cfg(not(target_arch = "wasm32"))]
            filename: None,
            #[cfg(not(target_arch = "wasm32"))]
            provider_watcher,
        };

        let provider_handler = ProviderEventHandler::new(snow.tree.clone());

        snow.event_handler.insert(
            EventKind::Provider,
            DynamicHandler::new::<ProviderEvent, Arc<Mutex<ProviderState<Message<AppMessage>>>>>(
                EventKind::Provider,
                provider_handler,
            ),
        );

        #[cfg(not(target_arch = "wasm32"))]
        let fsevent_handler = FsNotifyEventHandler::new(snow.tree.clone());

        #[cfg(not(target_arch = "wasm32"))]
        snow.event_handler.insert(
            EventKind::FsNotify,
            DynamicHandler::new::<notify::Event, Arc<Mutex<FsNotifyState<Message<AppMessage>>>>>(
                EventKind::FsNotify,
                fsevent_handler,
            ),
        );

        Ok(snow)
    }

    pub fn update_widgets(&self) {
        let mut pending = Vec::new();

        for node in self.tree.lock().as_ref().unwrap().root() {
            let _: Result<(), ()> = node.with_data(|inner| {
                if inner.widget.is_none() || inner.is_dirty() {
                    pending.push(node.clone());
                }
                Ok(())
            });
        }

        //info!("Pending Updates: {pending:?}");

        for _node in pending.into_iter().rev() {
            //DynamicWidget::from_node(node).unwrap();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn watch_tree_files(&mut self) -> Result<(), Error> {
        self.notify_state.lock().provider_map.clear();

        info!("Walking tree and adding files to watcher");

        for node in self.tree.lock().as_ref().unwrap().root() {
            match &**node.node().data() {
                Content::Value(value) => {
                    if let ValueKind::Dynamic {
                        provider: Some(provider),
                        ..
                    } = value.inner()
                    {
                        // Provide an event sender to this Provider
                        provider
                            .lock()
                            .set_event_inlet(self.event_endpoint.get_inlet());
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn init_watcher(mut inlet: Inlet<Event>) -> FsEventWatcher {
        // Create a filesystem watcher that writes events to a channel

        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                let event = match res {
                    Ok(fsevent) => Event::FsNotify(fsevent),
                    Err(e) => {
                        error!("Watcher error {e:#?}");
                        Event::FsNotifyError(e.to_string())
                    }
                };
                futures::executor::block_on(async {
                    inlet.send(event).await.unwrap();
                })
            },
            notify::Config::default(),
        )
        .unwrap();

        watcher
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_file(&mut self, filename: String) -> Result<(), Error> {
        let filename = &PathBuf::from(&filename);
        let tree = SnowcapParser::<Message<AppMessage>>::parse_file(&filename)?;

        let tree = IndexedTree::from_tree(tree);

        self.filename = Some(filename.clone());

        // Register the markup file with the watcher
        self.provider_watcher
            .watch(filename, notify::RecursiveMode::NonRecursive)?;

        //self.tree = Some(tree);

        self.set_tree(tree)?;

        // Register any files referenced by the markup with the watcher
        self.watch_tree_files()?;

        self.update_widgets();

        Ok(())
    }

    pub fn load_memory(&mut self, data: &str) -> Result<(), Error> {
        let tree = SnowcapParser::<Message<AppMessage>>::parse_memory(data)?;

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

    fn set_tree(&mut self, tree: IndexedTree<Message<AppMessage>>) -> Result<(), Error> {
        *self.tree.lock() = Some(tree);
        Ok(())
    }

    // Initial tasks to be executed in parallel by the iced Runtime
    pub fn init(&mut self) -> Task<Message<AppMessage>> {
        let mut tasks = Vec::new();

        // Start the event listener task
        tasks.push(Task::run(self.event_endpoint.take_outlet(), |ep_message| {
            info!("Received event from inlet {}", ep_message.from());
            Message::<AppMessage>::Event(ep_message.into_inner())
        }));

        // Run the initial tree update, and get any tasks (Provider init tasks)
        let tree_task = if let Some(tree) = &*self.tree.lock() {
            profiling::scope!("build-widgets");
            info!("{}", tree.root());
            match WidgetCache::update_tree(tree) {
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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn reload_file(&mut self) -> Result<(), Error> {
        use arbutus::{TreeDiff, TreeNode};
        use colored::Colorize;

        let filename = self.filename.clone().ok_or(Error::MissingAttribute(
            "No snowcap grammar filename in self".to_string(),
        ))?;

        // Parse the new file into an IndexedTree
        let mut new_tree =
            IndexedTree::from_tree(SnowcapParser::<Message<AppMessage>>::parse_file(&filename)?);

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
                    println!("TREE EVENT {event:?}");
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

    fn handle_widget_message(
        &mut self,
        node_id: &NodeId,
        message: &mut WidgetMessage,
    ) -> Task<Message<AppMessage>> {
        info!("Widget Message for NodeId: {node_id}");

        if let Some(node) = self.tree.lock().as_mut().unwrap().get_node_mut(node_id) {
            node.node_mut().data_mut().set_dirty(true);
        }

        match message {
            WidgetMessage::Markdown(url) => {
                info!("Markdown URL click {url}");
                Task::none()
            }
            WidgetMessage::Button(id) => {
                info!("Button clicked node={id:?}");
                Task::none()
            }
            WidgetMessage::Toggler { id, toggled } => {
                info!("Toggler node={id:?} toggled={toggled}");
                Task::none()
            }
            WidgetMessage::PickListSelected { id, selected } => {
                info!("Picklist selected node={id:?} selected={selected}");
                Task::none()
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Task<Message<AppMessage>> {
        let event_type: EventKind = (&event).into();
        if let Some(handler) = self.event_handler.get(&event_type) {
            debug!("Found handler {handler:#?}");
            let (module_event, module_state): (Box<dyn Any>, Box<dyn Any>) = match event {
                #[cfg(not(target_arch = "wasm32"))]
                Event::FsNotify(event) => (Box::new(event), Box::new(self.notify_state.clone())),
                Event::Provider(provider_event) => (
                    Box::new(provider_event),
                    Box::new(self.provider_state.clone()),
                ),
                _ => {
                    error!("Dynamic event handler was found, but missing event Box inner extraction pattern for {:?}", event);
                    panic!();
                }
            };

            debug!("Entering event handler");
            match handler.handle(module_event, module_state) {
                Ok(task) => {
                    debug!("Event handler returned");
                    task
                }
                Err(e) => {
                    error!("{e:?}");
                    Task::none()
                }
            }
        } else {
            // Event wasn't handled by dynamic dispatch
            match event {
                #[cfg(not(target_arch = "wasm32"))]
                Event::FsNotifyError(e) => {
                    error!("FsNotifyError {e:#?}");
                    Task::none()
                }
                #[cfg(not(target_arch = "wasm32"))]
                Event::WatchFileRequest { filename, provider } => {
                    info!("{provider:?} register {filename:?} with watcher");

                    match self
                        .provider_watcher
                        .watch(&filename, notify::RecursiveMode::NonRecursive)
                    {
                        Ok(_) => {
                            info!("Successfully added {filename:?} to watcher");

                            // Add the Provider to the map
                            self.notify_state
                                .lock()
                                .provider_map
                                .insert(filename.clone(), provider);
                        }
                        Err(e) => {
                            error!("Failed to add {filename:?} to watcher: {e:#?}")
                        }
                    }

                    Task::none()
                }

                _ => Task::none(),
            }
        }
    }

    #[profiling::function]
    pub fn update(&mut self, message: &mut Message<AppMessage>) -> Task<Message<AppMessage>> {
        let _message_type: MessageDiscriminants = (&*message).into();

        let task = {
            profiling::scope!("message-dispatch");
            let task = match message {
                Message::App(app_message) => {
                    debug!("AppMessage {app_message:?}");
                    //f(&app_message)
                    Task::none()
                }
                Message::Widget { node_id, message } => {
                    self.handle_widget_message(node_id, message)
                }
                Message::Event(event) => {
                    let event = std::mem::take(event);
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
            };
            task
        };

        let tree_task = if let Some(tree) = &*self.tree.lock() {
            profiling::scope!("build-widgets");
            info!("{}", tree.root());
            match WidgetCache::update_tree(tree) {
                Ok(task) => task,
                Err(e) => {
                    error!("Failed to build widgets: {}", e);
                    Task::none()
                }
            }
        } else {
            Task::none()
        };

        tree_task.chain(task)
    }

    #[profiling::function]
    pub fn view<'a>(&'a self) -> iced::Element<'a, Message<AppMessage>> {
        info!("View");

        let root = if let Some(tree) = &*self.tree.lock() {
            if let Some(widget) = &tree.root().node().data().widget {
                widget.clone().into_element().unwrap()
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
