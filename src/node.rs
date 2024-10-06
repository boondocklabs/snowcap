use std::collections::HashMap;

use tracing::{info, info_span, warn};

use crate::{
    parser::{ElementId, NodeId, TreeNode},
    Error,
};

#[derive(Debug)]
pub struct NodeManager<AppMessage> {
    nodes: HashMap<NodeId, TreeNode<AppMessage>>,
    elements: HashMap<ElementId, TreeNode<AppMessage>>,
}

impl<AppMessage> NodeManager<AppMessage>
where
    AppMessage: std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            elements: HashMap::new(),
        }
    }

    pub fn from_tree(root: TreeNode<AppMessage>) -> Result<Self, Error> {
        info_span!("NodeManager").in_scope(|| {
            info!("Creating NodeManager from tree");

            let mut manager = Self::new();

            // Find all DataSource nodes with DataProvider::File
            // and add them to the watcher
            for node in root {
                manager.insert_node(&node)?;

                if let Some(_id) = node.element_id() {
                    manager.insert_element(&node)?;
                }
            }
            Ok(manager)
        })
    }

    pub fn get_node(&mut self, node_id: &NodeId) -> Result<&mut TreeNode<AppMessage>, Error> {
        self.nodes
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(*node_id))
    }

    pub fn insert_node(&mut self, node: &TreeNode<AppMessage>) -> Result<(), Error> {
        if let Some(replaced) = self.nodes.insert(node.id().clone(), node.clone()) {
            panic!("Duplicate node {}. Already had {replaced:#?}", node.id());
        }
        info!("Added Node ID {}", node.id());
        Ok(())
    }

    pub fn insert_element(&mut self, node: &TreeNode<AppMessage>) -> Result<(), Error> {
        if let Some(id) = node.element_id() {
            if let Some(_replaced) = self.elements.insert(id.clone(), node.clone()) {
                warn!("Node {id} replaced. You may have duplicate IDs in your markup");
            }
            info!("Added Element {id:?}");
            Ok(())
        } else {
            Err(Error::MissingId)
        }
    }
}
