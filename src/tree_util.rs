use arbutus::{TreeNode, TreeNodeRef as _};
use iced::Element;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

use crate::{
    data::DataType, dynamic_widget::DynamicWidget, message::WidgetMessage,
    parser::value::ValueKind, ConversionError, IndexedTree, NodeId, NodeRef, Value,
};

#[derive(Debug)]
pub enum ChildData<'a, M> {
    Widget(DynamicWidget<'a, M>),
    Value(Value),
}

impl<'a, M> Into<Element<'a, M>> for ChildData<'a, M> {
    fn into(self) -> Element<'a, M> {
        match self {
            ChildData::Widget(dynamic_widget) => dynamic_widget.into_element(),
            ChildData::Value(value) => match &*value {
                ValueKind::String(_) => todo!(),
                ValueKind::Float(_) => todo!(),
                ValueKind::Integer(_) => todo!(),
                ValueKind::Boolean(_) => todo!(),
                ValueKind::Array(_vec) => todo!(),
                ValueKind::Dynamic { data, provider: _ } => match data {
                    Some(data) => match &**data {
                        DataType::Null => todo!(),
                        DataType::Image(_handle) => todo!(),
                        DataType::Svg(_handle) => todo!(),
                        DataType::QrCode(_arc) => todo!(),
                        DataType::Markdown(_markdown_items) => todo!(),
                        DataType::Text(_) => todo!(),
                    },
                    None => todo!(),
                },
            },
        }
    }
}

pub struct WidgetBuilder<M>
where
    M: 'static,
{
    /// Children widgets for the parent NodeId.
    /// Collected during widget construction from leaves downwards.
    children: HashMap<NodeId, HashMap<NodeId, ChildData<'static, M>>>,
}

impl<M> WidgetBuilder<M>
where
    M: std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            //widget_index: HashMap::new(),
            //widgets: Vec::new(),
            children: HashMap::new(),
        }
    }

    pub fn build_widgets(
        &mut self,
        tree: &IndexedTree<M>,
    ) -> Result<DynamicWidget<'static, M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        // Create the queue initialized with all leaf nodes of the tree
        let mut leaves = tree.leaves().clone();
        leaves.reverse();

        // Track which nodes have been visited, initialized with initial set of leaf nodes
        let mut visited: HashSet<NodeId> =
            leaves.iter().map(|leaf| leaf.node().id().clone()).collect();

        let mut queue: VecDeque<NodeRef<M>> = VecDeque::from(leaves);

        // Queue of nodes at the next depth. This will move into the outer queue
        // after it has drained indicating all nodes at the current depth have
        // been processed. This ensures all children are built before moving
        // to the next depth
        let mut next: VecDeque<NodeRef<M>> = VecDeque::new();

        while let Some(node) = queue.pop_front() {
            let node_id = node.node().id().clone();

            // Get the expected number of children for this node
            let expected_children = node.node().num_children();

            // Get the number of children we have in the cache for this node
            let have_children = self.children.get(&node_id).map(|v| v.len()).unwrap_or(0);

            if expected_children != have_children {
                // Put the node into the next depth queue, as not all dependencies
                // have been resolved yet.

                if queue.is_empty() {
                    queue.append(&mut next);
                }
                next.push_front(node);
                continue;
            }

            // Remove and take ownership of the children widgets of this node id, which were
            // built on the previous depth of this loop.
            let children = self.children.remove(&node_id);

            let child_data = match &node.node().data().data {
                crate::node::SnowcapNodeData::Container
                | crate::node::SnowcapNodeData::Widget(_)
                | crate::node::SnowcapNodeData::Row
                | crate::node::SnowcapNodeData::Column
                | crate::node::SnowcapNodeData::Stack => {
                    let widget =
                        DynamicWidget::builder(node.clone(), children)?.with_node_id(node_id);
                    Some(ChildData::Widget(widget))
                }
                crate::node::SnowcapNodeData::Value(value) => Some(ChildData::Value(value.clone())),
                crate::node::SnowcapNodeData::Root => {
                    // Root widget is in the children queue,
                    // built from the previous depth pass
                    Some(children.unwrap().drain().last().unwrap().1)
                }
                _ => None,
            };

            if let Some(parent) = node.node().parent() {
                let parent_node = parent.node();
                let parent_id = parent_node.id();

                // Collect children of parent nodes into self.children
                if let Some(child_data) = child_data {
                    self.children
                        .entry(parent_id)
                        .or_insert(HashMap::new())
                        .insert(node_id, child_data);
                }

                if visited.insert(parent_id) {
                    debug!("PUSH NEXT {parent_id}");
                    // Node has not been visited before, add to queue
                    next.push_front(parent.clone());
                }
            } else {
                // No parent. This is the root node.
                debug!("ROOT NODE");

                debug!("{child_data:?}");

                if let Some(ChildData::Widget(root)) = child_data {
                    return Ok(root);
                }
            }

            if queue.is_empty() {
                queue.append(&mut next);
            }
        }

        Err(ConversionError::Missing("no root".into()))
    }
}
