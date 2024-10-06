mod attribute;
mod connector;
mod conversion;
mod data;
mod error;
mod message;
mod node;
mod parser;

use connector::{Endpoint, Inlet};
use data::file_provider::FileData;
use data::provider::Provider;
use data::DataType;
use data::MarkdownItems;
pub use iced;
use iced::advanced::graphics::futures::MaybeSend;
use message::Event;
use node::NodeManager;
use parking_lot::Mutex;
use parser::TreeNode;
use tracing::warn;

use std::collections::HashMap;
use std::path::PathBuf;
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

pub use parser::MarkupTree;
pub use parser::SnowcapParser;
pub use parser::Value;

use tracing::debug;
use tracing::error;
use tracing::info;

#[derive(Debug)]
pub struct Snowcap<AppMessage> {
    #[cfg(not(target_arch = "wasm32"))]
    filename: Option<PathBuf>,

    node_manager: NodeManager<AppMessage>,
    tree: Option<TreeNode<AppMessage>>,

    #[cfg(not(target_arch = "wasm32"))]
    watcher: FsEventWatcher,

    //event_tx: UnboundedSender<Event>,
    //event_rx: Option<UnboundedReceiver<Event>>,
    event_endpoint: Endpoint<Event>,

    provider_map: HashMap<PathBuf, Arc<Mutex<dyn Provider + 'static>>>,
}

impl<AppMessage> Snowcap<AppMessage>
where
    AppMessage: Clone + MaybeSend + std::fmt::Debug + 'static,
{
    pub fn watch_tree_files(&mut self) {
        self.provider_map.clear();

        info!("Walking tree and adding files to watcher");

        for node in self.tree.as_ref().unwrap().clone() {
            if let MarkupTree::Value(value) = node.inner() {
                if let Value::Data {
                    provider: Some(provider),
                    ..
                } = &*value.borrow()
                {
                    // Provide an event sender to this Provider
                    provider
                        .lock()
                        .set_event_inlet(self.event_endpoint.get_inlet());

                    /*
                    // If this is a file provider, register the file with the watcher
                    if let DataProvider::File(file_provider) = &**provider {
                        let path = file_provider.path();
                        info!("Found DataSource FileProvider for file {:?}", path);

                        // Store an Arc of the FileProvider in the hashmap
                        self.provider_map.insert(path.into(), (*provider).clone());

                        // Add the path to the watcher
                        self.watcher
                            .watch(path, notify::RecursiveMode::NonRecursive)
                            .unwrap();
                    }
                    */
                }
            }
        }
    }

    pub fn get_provider_tasks(&self) -> Vec<Task<Message<AppMessage>>> {
        let mut tasks: Vec<Task<Message<AppMessage>>> = Vec::new();

        for node in self.tree.as_ref().unwrap().clone() {
            if let MarkupTree::Value(value) = node.inner() {
                if let Value::Data {
                    provider: Some(provider),
                    ..
                } = &*value.borrow()
                {
                    tasks.push(
                        provider
                            .lock()
                            .init_task(provider.clone(), node.id())
                            .map(|e| Message::Event(e))
                            .into(),
                    );
                }
            }
        }

        tasks
    }

    pub fn new() -> Result<Self, Error> {
        let event_endpoint = Endpoint::new();

        info!("Event Endpoint ID: {}", event_endpoint.id());

        #[cfg(not(target_arch = "wasm32"))]
        // Initialize the filesystem watcher
        //let watcher = Self::init_watcher(event_tx.clone());
        let watcher = Self::init_watcher(event_endpoint.get_inlet());

        let snow = Self {
            #[cfg(not(target_arch = "wasm32"))]
            filename: None,
            node_manager: NodeManager::new(),
            tree: None,
            #[cfg(not(target_arch = "wasm32"))]
            watcher,
            //event_tx,
            //event_rx: Some(event_rx),
            event_endpoint,
            provider_map: HashMap::new(),
        };

        Ok(snow)
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

    pub fn load_file(&mut self, filename: String) -> Result<(), Error> {
        let filename = &PathBuf::from(&filename);
        let tree = SnowcapParser::parse_file(&filename)?;

        self.filename = Some(filename.clone());

        // Register the markup file with the watcher
        self.watcher
            .watch(filename, notify::RecursiveMode::NonRecursive)?;

        // Build a NodeManager from the AST.
        // This maps any explicitly defined NodeId's from the markup to TreeNode references
        self.node_manager = NodeManager::from_tree(tree.clone())?;

        self.tree = Some(tree);

        // Register any files referenced by the markup with the watcher
        self.watch_tree_files();

        Ok(())
    }

    /*
    pub fn from_memory(data: &str) -> Result<Self, Error> {
        let tree = SnowcapParser::parse_memory(data)?;

        Ok(Self {
            #[cfg(not(target_arch = "wasm32"))]
            filename: None,
            node_manager: NodeManager::from_tree(tree.clone())?,
            tree,
            #[cfg(not(target_arch = "wasm32"))]
            watcher: None,
            #[cfg(not(target_arch = "wasm32"))]
            watcher_rx: None,

            watch_nodes: HashMap::new(),
        })
    }
    */

    pub fn root(&self) -> TreeNode<AppMessage> {
        self.tree.as_ref().unwrap().clone()
    }

    // Initial tasks to be executed in parallel by the iced Runtime
    pub fn init(&mut self) -> Task<Message<AppMessage>> {
        let mut tasks = Vec::new();

        let mut provider_tasks = self.get_provider_tasks();

        tasks.append(&mut provider_tasks);

        // Start the event listener task
        tasks.push(Task::run(self.event_endpoint.take_outlet(), |ep_message| {
            //tracing::info_span!("forwarder").in_scope(|| {
            info!("Received event from inlet {}", ep_message.from());
            Message::<AppMessage>::Event(ep_message.into_inner())
            //})
        }));

        tasks.push(Task::perform(async { true }, |_event| {
            Message::Event(Event::Debug("Sending...".to_string()))
        }));

        Task::batch(tasks)
    }

    pub fn reload_file(&mut self) -> Result<(), Error> {
        let filename = self.filename.clone().ok_or(Error::MissingAttribute(
            "No snowcap grammar filename in self".to_string(),
        ))?;
        let tree = SnowcapParser::parse_file(&filename)?;
        self.node_manager = NodeManager::from_tree(tree.clone())?;
        self.tree = Some(tree);

        self.watch_tree_files();

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Task<Message<AppMessage>> {
        match event {
            Event::Debug(msg) => {
                info!("Received debug message {msg}");
                Task::none()
            }
            #[cfg(not(target_arch = "wasm32"))]
            Event::FsNotify(event) => match event.kind {
                notify::EventKind::Modify(notify::event::ModifyKind::Data(_change)) => {
                    let mut tasks: Vec<Task<Message<AppMessage>>> = Vec::new();
                    for path in &event.paths {
                        info!("File change notification for {path:?}");

                        // Find the provider of this file path from the provider map
                        if let Some(provider) = self.provider_map.get(path) {
                            // Get the update task for this Provider
                            let task = provider.lock().update_task().map(Message::Event);
                            tasks.push(task);
                        } else {
                            // Since we didn't find the path in the map of nodes
                            // which reference the changed file, we can assume
                            // that this is the markup file itself that has changed.
                            if let Err(e) = self.reload_file() {
                                error!("{e:#?}");
                            }

                            // Kickoff the providers
                            let mut provider_tasks = self.get_provider_tasks();
                            tasks.append(&mut provider_tasks);
                        }
                    }
                    Task::batch(tasks)
                }
                _ => Task::none(),
            },
            Event::FsNotifyError(e) => {
                error!("FsNotifyError {e:#?}");
                Task::none()
            }
            Event::Provider(provider_event) => {
                match provider_event {
                    data::provider::ProviderEvent::Updated => todo!(),
                    data::provider::ProviderEvent::FileLoaded { node_id, data } => {
                        info!("File Loaded. Node ID: {node_id} {data:?}");
                        match self.node_manager.get_node(&node_id) {
                            Ok(node) => {
                                if let MarkupTree::Value(value) = node.inner() {
                                    match data {
                                        data::file_provider::FileData::Svg(handle) => {
                                            let mut val = value.borrow_mut();
                                            match &mut *val {
                                                Value::Data { data, .. } => {
                                                    data.replace(Arc::new(DataType::Svg(handle)));
                                                }
                                                _ => panic!("Expecting Value::Data"),
                                            }
                                        }

                                        data::file_provider::FileData::Image(handle) => {
                                            let mut val = value.borrow_mut();
                                            match &mut *val {
                                                Value::Data { data, .. } => {
                                                    data.replace(Arc::new(DataType::Image(handle)));
                                                }
                                                _ => panic!("Expecting Value::Data"),
                                            }
                                        }
                                        data::file_provider::FileData::Text(text) => {
                                            let mut val = value.borrow_mut();
                                            match &mut *val {
                                                Value::Data { data, .. } => {
                                                    data.replace(Arc::new(DataType::Text(text)));
                                                }
                                                _ => panic!("Expecting Value::Data"),
                                            }
                                        }
                                        data::file_provider::FileData::Markdown(items) => {
                                            let mut val = value.borrow_mut();
                                            match &mut *val {
                                                Value::Data { data, .. } => {
                                                    data.replace(Arc::new(DataType::Markdown(
                                                        MarkdownItems::new(items),
                                                    )));
                                                }
                                                _ => panic!("Expecting Value::Data"),
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => todo!(),
                        }
                    }
                    data::provider::ProviderEvent::UrlLoaded { node_id, url, data } => {
                        info!("URL Loaded {url:?} {data:?}");
                        match self.node_manager.get_node(&node_id) {
                            Ok(node) => {
                                if let MarkupTree::Value(value) = node.inner() {
                                    match data {
                                        data::file_provider::FileData::Text(text) => {
                                            match &mut *value.borrow_mut() {
                                                Value::Data { data, .. } => {
                                                    data.replace(Arc::new(DataType::Text(text)));
                                                }
                                                _ => panic!("Expecting Value::Data"),
                                            }
                                        }
                                        FileData::Image(handle) => match &mut *value.borrow_mut() {
                                            Value::Data { data, .. } => {
                                                data.replace(Arc::new(DataType::Image(handle)));
                                            }
                                            _ => panic!("Expecting Value::Data"),
                                        },
                                        _ => error!("Tree FileData type not supported"),
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Node {node_id:?}: {e:?}");
                            }
                        }
                    }
                    data::provider::ProviderEvent::Error(_) => todo!(),
                }

                Task::none()
            }
            Event::Empty => todo!(),
            Event::WatchFileRequest { filename, provider } => {
                info!("{provider:?} register {filename:?} with watcher");

                match self
                    .watcher
                    .watch(&filename, notify::RecursiveMode::NonRecursive)
                {
                    Ok(_) => {
                        info!("Successfully added {filename:?} to watcher");

                        // Add the Provider to the map
                        self.provider_map.insert(filename.clone(), provider);
                    }
                    Err(e) => {
                        error!("Failed to add {filename:?} to watcher: {e:#?}")
                    }
                }

                Task::none()
            }
        }
    }

    pub fn update(&mut self, message: &mut Message<AppMessage>) -> Task<Message<AppMessage>> {
        match message {
            Message::App(app_message) => {
                debug!("AppMessage {app_message:?}");
                //f(&app_message)
                Task::none()
            }
            Message::Markdown(url) => {
                info!("Markdown URL click {url}");
                Task::none()
            }
            Message::Button(id) => {
                info!("Button clicked node={id:?}");
                Task::none()
            }
            Message::Toggler { id, toggled } => {
                info!("Toggler node={id:?} toggled={toggled}");
                Task::none()
            }
            Message::PickListSelected { id, selected } => {
                info!("Picklist selected node={id:?} selected={selected}");
                Task::none()
            }
            Message::Event(event) => {
                let event = std::mem::take(event);
                self.handle_event(event)
            }
            Message::Empty => {
                warn!("Update called on Empty Message");
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message<AppMessage>> {
        info!("View");

        if let Some(root) = &self.tree {
            let guard = root.element_read();
            if let Some(_element) = &*guard {
                //let w = element.as_widget();
                //return iced::Element::new(*element.as_widget());
            }
        }

        match self.tree.as_ref().unwrap().try_into() {
            Ok(content) => content,
            Err(e) => {
                error!("{e:?}");
                iced::widget::text(format!("{e:?}")).into()
            }
        }
    }
}
