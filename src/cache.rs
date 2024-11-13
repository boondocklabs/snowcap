//! In-tree widget cache and Tree widget updates

use std::{collections::HashMap, time::Instant};

use arbutus::{TreeNode, TreeNodeRef as _};
use colored::Colorize as _;
use iced::{Element, Task};
use salish::Message;
use tracing::{debug, debug_span, instrument};

use crate::{
    attribute::Attributes,
    conversion::{
        column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, stack::SnowcapStack,
        widget::SnowcapWidget,
    },
    dynamic_widget::DynamicWidget,
    module::{data::ModuleData, manager::ModuleManager},
    node::{Content, SnowcapNode, State},
    parser::module::Module,
    ConversionError, IndexedTree, NodeId, NodeRef, Value,
};

/// Widget content passed to the widget builders to provide their content.
#[derive(Debug)]
pub enum WidgetContent<M> {
    None,
    Widget(DynamicWidget<M>),
    Value(Value),
    List(Vec<Self>),
    Module(Module),
    Text(String),
    Image(iced::widget::image::Handle),
    Svg(iced::widget::svg::Handle),
}

impl<M> std::fmt::Display for WidgetContent<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetContent::None => write!(f, "None"),
            WidgetContent::Widget(dynamic_widget) => write!(f, "{:?}", dynamic_widget),
            WidgetContent::Value(value) => write!(f, "{}", value),
            WidgetContent::List(vec) => {
                write!(f, "[")?;
                for w in vec {
                    write!(f, "{:?},", w)?;
                }
                write!(f, "]")
            }
            WidgetContent::Module(module) => write!(f, "{module}"),
            WidgetContent::Image(_) => write!(f, "Image Handle"),
            WidgetContent::Svg(_) => write!(f, "SVG Handle"),
            WidgetContent::Text(_) => write!(f, "Text Content"),
            //WidgetContent::Markdown(_) => write!(f, "Markdown Items"),
        }
    }
}

/// Conversion of a reference to a boxed dyn [`ModuleData`] into [`WidgetContent`]
impl<M> From<&Box<dyn ModuleData>> for WidgetContent<M> {
    fn from(data: &Box<dyn ModuleData>) -> Self {
        match data.kind() {
            crate::module::data::ModuleDataKind::Unknown => todo!(),
            crate::module::data::ModuleDataKind::Image => WidgetContent::Image(
                iced::widget::image::Handle::from_bytes(data.bytes().unwrap().clone()),
            ),
            crate::module::data::ModuleDataKind::Svg => WidgetContent::Svg(
                iced::widget::svg::Handle::from_memory(data.bytes().unwrap().clone()),
            ),
            crate::module::data::ModuleDataKind::Text => {
                WidgetContent::Text(String::from_utf8(data.bytes().unwrap().clone()).unwrap())
            }
        }
    }
}

/// Convert WidgetContent into iced::Element
impl<M> Into<Element<'static, M>> for WidgetContent<M>
where
    M: 'static,
{
    fn into(self) -> Element<'static, M> {
        match self {
            WidgetContent::Widget(dynamic_widget) => dynamic_widget.into_element().unwrap(),
            _ => unimplemented!("Cannot convert non-Widget type content into an Element"),
        }
    }
}

/// Convert WidgetContent into an Iterator which yields iced::Elements
impl<M> IntoIterator for WidgetContent<M>
where
    M: std::fmt::Debug + 'static,
{
    type Item = Element<'static, M>;

    type IntoIter = std::vec::IntoIter<Element<'static, M>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            WidgetContent::Widget(widget) => vec![widget.into_element().unwrap()].into_iter(),
            WidgetContent::List(vec) => vec
                .into_iter()
                .filter_map(|item| {
                    if let WidgetContent::Widget(widget) = item {
                        Some(widget.into_element().unwrap())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .into_iter(),
            _ => {
                tracing::warn!("Attempt to iterate non iterable type {:?}", self);
                Vec::new().into_iter()
            } // Return an empty iterator for non-iterable types
        }
    }
}

/// Cache of Widgets and tree updates
#[derive(Default, Debug)]
pub struct WidgetCache {
    widgets: HashMap<NodeId, DynamicWidget<Message>>,
}

impl WidgetCache {
    #[instrument("cache")]
    pub fn drop_widget(&mut self, node_id: NodeId) {
        debug!("Dropping widget {node_id}");
        self.widgets.remove(&node_id);
    }

    /// Get the cached widget for the specified NodeId, or None
    /// if it doesn't exist in the cache
    pub fn get(&self, node_id: NodeId) -> Option<DynamicWidget<Message>> {
        self.widgets.get(&node_id).cloned()
    }

    /// Find dirty paths, mark nodes as dirty along the path and drop widgets.
    ///
    /// This must be done in its own scope so the RwLock write guards in WidgetRef are released.
    /// The parent of a node holds its WidgetRef in the contents of the parent,
    /// so we drop all widgets along the path before rebuilding.
    #[profiling::function]
    fn mark_dirty_paths(
        &mut self,
        tree: &IndexedTree,
        modules: &mut ModuleManager,
    ) -> Result<(Vec<NodeRef>, Vec<Task<Message>>), ConversionError>
where
        //M: std::fmt::Debug + From<WidgetMessage> + From<ModuleMessage> + MaybeSend + 'static,
    {
        let start = Instant::now();

        // Nodes which need updates
        let mut update_queue: Vec<NodeRef> = Vec::new();
        let mut tasks: Vec<Task<Message>> = Vec::new();

        debug!("Start marking dirty paths");

        // The leaf iterator yields nodes in descending order from the leaves,
        // always yielding children of parents first, and the root node
        // is always last. Pushing nodes into the queue and rebuilding them will thus be
        // in the correct order ensuring all children widgets are built and cached
        // before their parents.
        tree.leaf_iter().for_each(|noderef| {
            let mut node = noderef.node_mut();

            debug!("Node {} state={:?}", node.id(), node.data().get_state());

            match node.data().get_state() {
                State::New => {
                    let data = node.data_mut();
                    // Check if this node is a Module, and instantiate the module
                    if let Content::Module(module) = data.content_mut() {
                        let args = module.args().clone();

                        // Instantate the module, and get its handle_id and init task
                        let (handle_id, task) =
                            modules.instantiate(module.name(), module.args().clone())?;

                        // Set the Handle ID of the instantiated module into the tree node
                        module.set_handle_id(handle_id);

                        // Connect a NodeRef to the module
                        modules.connect_node(handle_id, noderef.clone());

                        // Push the update task from the module to the set of tasks to run
                        // after this update pass has completed.
                        tasks.push(task);

                        println!(
                            "Instantiated module handle {handle_id} for node {} args {}",
                            node.id().clone(),
                            args
                        );
                    }

                    drop(node);
                    update_queue.push(noderef.clone());
                }
                State::Dirty => {
                    debug!(
                        "Dirty Node id={} data={}. Dropping widget.",
                        node.id(),
                        node.data()
                    );
                    self.drop_widget(node.id());
                    //drop(node.data_mut().widget.take());

                    // Mark the parent widget as dirty
                    if let Some(parent) = node.parent_mut() {
                        parent.node_mut().data_mut().set_dirty(true);
                    }

                    // Push this noderef into the update queue
                    drop(node);
                    update_queue.push(noderef.clone())
                }

                // Ignore clean nodes
                State::Clean => {}
            }

            // We can propagate errors out of the closure, but must return Ok(()) to continue the iterator
            Ok::<(), ConversionError>(())
        })?;

        let duration = Instant::now() - start;
        debug!("Finished marking dirty paths. Took {duration:?}");
        Ok((update_queue, tasks))
    }

    /// Collect cached [`DynamicWidget`] objects for all children of this node, if there are any.
    /// Returns None if no cached widgets are available.
    fn child_widgets(&self, node: &NodeRef) -> Option<Vec<DynamicWidget<Message>>> {
        let node = node.node();

        let child_widgets: Option<Vec<DynamicWidget<Message>>> =
            node.children().and_then(|children| {
                let widgets: Vec<DynamicWidget<Message>> = children
                    .iter()
                    //.filter_map(|child| child.node().data().widget.clone())
                    .filter_map(|child| self.widgets.get(&child.node().id()).cloned())
                    .collect();

                (!widgets.is_empty()).then_some(widgets)
            });
        child_widgets
    }

    /// Get [`WidgetContent`] for a node from a Vec of [`DynamicWidget`] of the children
    fn widget_content(
        noderef: &NodeRef,
        child_widgets: Option<Vec<DynamicWidget<Message>>>,
    ) -> WidgetContent<Message> {
        let node = noderef.node();

        let content = if let Some(mut children) = child_widgets {
            // Theis node has children with widgets
            if children.len() == 1 {
                WidgetContent::Widget(children.pop().unwrap())
            } else {
                WidgetContent::List(
                    children
                        .into_iter()
                        .map(|w| WidgetContent::Widget(w))
                        .collect(),
                )
            }
        } else {
            // This node does not have childlren with widgets. Could be a Module or a Value
            if node.num_children() == 1 {
                let child = node.children().unwrap().iter().last().unwrap();

                match child.node().data().content() {
                    Content::Value(value) => WidgetContent::Value(value.clone()),
                    Content::Module(module) => {
                        if let Some(data) = child.node().data().module_data() {
                            WidgetContent::from(data)
                        } else {
                            WidgetContent::Module(module.clone())
                        }
                    }
                    _ => WidgetContent::None,
                }
            } else if node.num_children() > 1 {
                println!("{}\n{noderef}", "At Node".red());
                panic!("More than one child node for a node with data content");
            } else {
                WidgetContent::None
            }
        };

        content
    }

    /// Build the widget for a Node
    fn build_widget(
        node_id: NodeId,
        attrs: Attributes,
        data: &SnowcapNode,
        content: WidgetContent<Message>,
    ) -> Result<Option<DynamicWidget<Message>>, ConversionError> {
        let widget = match &**data {
            Content::Widget(widget) => {
                debug!("Building widget {widget} node {node_id} contents {content}");

                let widget = SnowcapWidget::new(
                    node_id,
                    widget.clone(),
                    data.element_id.clone(),
                    attrs,
                    content,
                )?
                .with_node_id(node_id);

                Some(widget)
            }
            Content::Container => {
                debug!("Building Container node {node_id} contents {content}");
                let widget = SnowcapContainer::new(attrs, content)?.with_node_id(node_id);
                Some(widget)
            }
            Content::Row => {
                debug!("Building Row node {node_id} contents {content}");
                let widget = SnowcapRow::convert(attrs, content)?.with_node_id(node_id);
                Some(widget)
            }
            Content::Column => {
                debug!("Building Column node {node_id} contents {content}");
                let widget = SnowcapColumn::convert(attrs, content)?.with_node_id(node_id);
                Some(widget)
            }
            Content::Stack => {
                debug!("Building Stack node {node_id} contents {content}");
                let widget = SnowcapStack::convert(attrs, content)?.with_node_id(node_id);
                Some(widget)
            }
            Content::Root => {
                debug!("Building Root node {node_id} contents {content}");
                if let WidgetContent::Widget(widget) = content {
                    Some(widget)
                } else {
                    panic!("No widget in root");
                }
            }
            Content::Module(_module) => None,
            Content::Value(_value) => None,
            Content::None => None,
        };

        Ok(widget)
    }

    /// Perform updates to widgets in the tree
    #[profiling::function]
    pub fn update_tree(
        &mut self,
        tree: &IndexedTree,
        module_manager: &mut ModuleManager,
    ) -> Result<Task<Message>, ConversionError>
where
        /*
        M: Clone
            + std::fmt::Debug
            //+ From<WidgetMessage>
            //+ From<ModuleMessage>
            + MaybeSend
            + 'static,
        */ {
        let start = Instant::now();

        debug_span!("tree-update").in_scope(|| {
            // First pass - Find dirty paths, mark nodes along the paths as dirty, and drop cached widgets
            let (queue, tasks) = self.mark_dirty_paths(tree, module_manager)?;

            for noderef in queue {
                let node = noderef.try_node()?;
                let data = node.data();
                let node_id = node.id();
                let attrs = data.attrs.clone();

                if self.widgets.contains_key(&node_id) {
                    // Already have a widget for this node, continue down the tree
                    return Ok(Task::none());
                }

                // Get a Vec of the children's DynamicWidgets
                let child_widgets = self.child_widgets(&noderef);

                // Get the WidgetContent for this node
                let content = Self::widget_content(&noderef, child_widgets);

                let widget = Self::build_widget(node_id, attrs, data, content)?;

                // Drop node so we can reborrow as mutable
                drop(node);

                if let Some(widget) = widget {
                    // Replace the widget
                    self.widgets.insert(node_id, widget);
                    //noderef.try_node_mut()?.data_mut().widget.replace(widget);
                }

                // Mark the node as clean
                noderef.try_node_mut()?.data_mut().set_state(State::Clean);
            }

            let duration = Instant::now() - start;
            debug!("Finished updating tree. Took {duration:?}");

            Ok(Task::batch(tasks))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Source;
    use tracing_test::traced_test;

    use crate::{cache::WidgetCache, module::manager::ModuleManager, Message, SnowcapParser};

    #[traced_test]
    #[test]
    pub fn build_tree_widgets() {
        let tree =
            SnowcapParser::<Message>::parse_memory(r#"{-[text("A"), text("B"), text("C")]}"#)
                .unwrap()
                .index();

        println!("{}", tree.root());

        let router =
            salish::router::MessageRouter::<iced::Task<salish::message::Message>, Source>::new();

        let mut modules = ModuleManager::new(router);

        let mut cache = WidgetCache::default();

        let _task = cache.update_tree(&tree, &mut modules).unwrap();
    }
}
