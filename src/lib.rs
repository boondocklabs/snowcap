mod attribute;
mod cache;
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

use arbutus::Node as _;
use arbutus::NodeRef as _;
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
pub use iced;
use iced::advanced::graphics::futures::MaybeSend;
use iced::widget::Text;
use message::Command;
use message::Event;
use message::EventKind;
use message::MessageDiscriminants;
use message::WidgetMessage;
use node::SnowcapNode;
use node::SnowcapNodeData;
use parking_lot::Mutex;
use tracing::warn;
use tree_util::WidgetBuilder;

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

pub use conversion::theme::SnowcapTheme;
pub use error::*;
use iced::futures;
use iced::futures::SinkExt;
use iced::Task;
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

type Node<Data, Id> = arbutus::TreeNodeRefCell<Data, Id>;
type NodeRef<M> = arbutus::NodeRefRc<Node<SnowcapNode<M>, arbutus::NodeId>>;

//type Node<Data, Id> = arbutus::TreeNodeSimple<Data, Id>;
//type NodeRef<M> = arbutus::NodeRefRef<Node<SnowcapNode<M>, arbutus::NodeId>>;

type Tree<M> = arbutus::Tree<NodeRef<M>>;
type IndexedTree<M> = arbutus::IndexedTree<NodeRef<M>>;
type NodeId = arbutus::NodeId;

#[derive(Debug)]
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
            #[cfg(not(target_arch = "wasm32"))]
            filename: None,
            tree,
            #[cfg(not(target_arch = "wasm32"))]
            provider_watcher,
            event_endpoint,
            event_handler: HashMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            notify_state,
            provider_state,
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
                if inner.widget.is_none() || inner.dirty == true {
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
                SnowcapNodeData::Value(value) => {
                    if let Value::Dynamic {
                        provider: Some(provider),
                        ..
                    } = value
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

    pub fn get_provider_tasks(&self) -> Vec<Task<Message<AppMessage>>> {
        let mut tasks: Vec<Task<Message<AppMessage>>> = Vec::new();

        if let Some(tree) = &*self.tree.as_ref().lock() {
            for node in tree.root() {
                let _: Result<(), ()> = node.with_data(|inner| {
                    match &inner.data {
                        SnowcapNodeData::Value(value) => {
                            if let Value::Dynamic {
                                provider: Some(provider),
                                ..
                            } = value
                            {
                                let task = provider
                                    .lock()
                                    .init_task(provider.clone(), node.node().id().clone())
                                    .map(|e| Message::<AppMessage>::Event(e))
                                    .into();

                                info!("Pushing provider init task for {provider:?}");
                                tasks.push(task)
                            }
                        }
                        _ => {}
                    }
                    Ok(())
                });
            }
        }
        tasks
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

        let mut provider_tasks = self.get_provider_tasks();

        tasks.append(&mut provider_tasks);

        // Start the event listener task
        tasks.push(Task::run(self.event_endpoint.take_outlet(), |ep_message| {
            info!("Received event from inlet {}", ep_message.from());
            Message::<AppMessage>::Event(ep_message.into_inner())
        }));

        tasks.push(Task::perform(async { true }, |_event| {
            Message::Event(Event::Debug("Sending...".to_string()))
        }));

        info!("Starting init tasks");

        Task::batch(tasks)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn reload_file(&mut self) -> Result<(), Error> {
        let filename = self.filename.clone().ok_or(Error::MissingAttribute(
            "No snowcap grammar filename in self".to_string(),
        ))?;
        self.set_tree(IndexedTree::from_tree(
            SnowcapParser::<Message<AppMessage>>::parse_file(&filename)?,
        ))?;
        //self.watch_tree_files();
        Ok(())
    }

    fn handle_widget_message(
        &mut self,
        node_id: &NodeId,
        message: &mut WidgetMessage,
    ) -> Task<Message<AppMessage>> {
        info!("Widget Message for NodeId: {node_id}");

        if let Some(node) = self.tree.lock().as_mut().unwrap().get_node_mut(node_id) {
            node.node_mut().data_mut().dirty = true;
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

            return handler.handle(module_event, module_state).unwrap();
        } else {
            // Event wasn't handled by dynamic dispatch
            match event {
                Event::Debug(msg) => {
                    info!("Received debug message {msg}");
                    Task::none()
                }
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

    pub fn update(&mut self, message: &mut Message<AppMessage>) -> Task<Message<AppMessage>> {
        let _message_type: MessageDiscriminants = (&*message).into();

        match message {
            Message::App(app_message) => {
                debug!("AppMessage {app_message:?}");
                //f(&app_message)
                Task::none()
            }
            Message::Widget { node_id, message } => self.handle_widget_message(node_id, message),
            Message::Event(event) => {
                let event = std::mem::take(event);
                self.handle_event(event)
            }
            Message::Empty => {
                warn!("Update called on Empty Message");
                Task::none()
            }

            Message::Command(Command::Reload) => {
                if let Err(e) = self.reload_file() {
                    error!("Failed to reload markup file: {e:#?}");
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self) -> iced::Element<'a, Message<AppMessage>> {
        info!("View");

        if let Some(tree) = &*self.tree.lock() {
            info!("{}", tree.root());
            let mut builder = WidgetBuilder::new();
            match builder.build_widgets(tree) {
                Ok(root) => {
                    return root.into_element();
                }
                Err(e) => {
                    error!("{:#?}", e);
                }
            }
        }

        /*
        self.update_widgets();

        if let Some(tree) = &*self.tree.lock() {
            let mut iter = tree.root().into_iter();
            iter.next();
            let root = iter.next().unwrap();

            let node = root.node();
            let data = node.data();

            let ele = data.as_element();

            if ele.is_ok() {
                return ele.unwrap();
            } else {
                Text::new("No root widget").into()
            }
            */

        /*
            let res = root.with_data(|node| match node.data {
                SnowcapNodeData::Container => match DynamicWidget::from_node(root.clone()) {
                    Ok(widget) => Ok(widget.into_element()),
                    Err(e) => {
                        error!("{e:#?}");
                        Err(e)
                    }
                },
                _ => Ok(Text::new("Expecting Container root node").into()),
            });

            match res {
                Ok(element) => element,
                Err(e) => Text::new(format!("{e:#?}")).into(),
            }
        } else {
            iced::widget::Text::new("No tree defined").into()
        }
        */
        iced::widget::Text::new("No tree defined").into()
    }
}
