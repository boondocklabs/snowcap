use std::{borrow::Borrow, collections::HashMap};

use tracing::{info, info_span, warn};

use crate::{
    message::WidgetMessage,
    parser::ElementId,
    tree::node::{NodeId, TreeNode},
    Error,
};

#[derive(Debug)]
pub struct NodeManager<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    nodes: HashMap<NodeId, TreeNode<'a, M>>,
    elements: HashMap<ElementId, TreeNode<'a, M>>,
}

impl<'a, M> NodeManager<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage>,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            elements: HashMap::new(),
        }
    }

    /*
    pub fn from_tree(root: T) -> Result<Self, Error>
    where
        T: IntoIterator<Item = T> + Clone,
    {
        info_span!("NodeManager").in_scope(|| {
            info!("Creating NodeManager from tree");

            let mut manager = Self::new();

            for node in root {
                manager.insert_node(node.clone())?;

                if let Some(_id) = node.borrow().element_id() {
                    manager.insert_element(node)?;
                }
            }
            Ok(manager)
        })
    }
    */

    pub fn update_from_tree(&mut self, root: &TreeNode<'a, M>) -> Result<(), Error> {
        info_span!("NodeManager").in_scope(|| {
            info!("Updating node manager from tree");

            for node in root {
                self.insert_node(node.clone())?;

                if let Some(_id) = node.inner.as_ref().borrow().get_element_id() {
                    self.insert_element(node.clone())?;
                }
            }

            Ok(())
        })
    }

    pub fn get_node(&self, node_id: NodeId) -> Result<TreeNode<'a, M>, Error> {
        if let Some(node) = self.nodes.get(&node_id) {
            Ok(node.clone())
        } else {
            Err(Error::NodeNotFound(node_id))
        }
    }

    pub fn insert_node(&mut self, node: TreeNode<'a, M>) -> Result<(), Error> {
        info!("Adding Node ID {}", node.borrow().id());
        if let Some(_replaced) = self.nodes.insert(node.borrow().id().clone(), node) {
            panic!("Attempted to insert duplicate node ID",);
        }
        Ok(())
    }

    pub fn insert_element(&mut self, node: TreeNode<'a, M>) -> Result<(), Error> {
        let id = {
            let inner = node.inner.as_ref().borrow();
            inner.get_element_id().clone()
        };

        //let id = node.inner().get_element_id().clone();
        if let Some(id) = id {
            if let Some(_replaced) = self.elements.insert(id.clone(), node) {
                warn!("Node {id} replaced. You may have duplicate IDs in your markup");
            }
            info!("Added Element {id:?}");
            Ok(())
        } else {
            Err(Error::MissingId)
        }
    }
}
