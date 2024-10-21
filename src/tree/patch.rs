use arbutus::{Node, NodeRef};
use tracing::{debug, debug_span, error};

use crate::{attribute::Attributes, node::SnowcapNodeData, IndexedTree, NodeId};

#[derive(Debug, Clone)]
pub enum PatchOperation {
    SetAttributes(Option<Attributes>),
    SetData(SnowcapNodeData),
}

#[derive(Debug)]
pub struct TreePatch {
    node_id: NodeId,
    operation: PatchOperation,
}

#[allow(dead_code)]
impl TreePatch {
    pub fn new(node_id: NodeId, operation: PatchOperation) -> Self {
        Self { node_id, operation }
    }

    pub fn operation(&self) -> &PatchOperation {
        &self.operation
    }
}

pub struct TreePatcher;

impl TreePatcher {
    pub fn patch<M>(tree: &mut IndexedTree<M>, patches: Vec<TreePatch>)
    where
        M: std::fmt::Debug,
    {
        debug_span!("TreePatcher").in_scope(|| {
            for patch in patches {
                // Find the node to patch
                if let Some(node) = tree.get_node_mut(&patch.node_id) {
                    match patch.operation {
                        PatchOperation::SetAttributes(attributes) => {
                            debug!("Patching attributes in node {}", patch.node_id);
                            node.node_mut().data_mut().attrs = attributes;
                        }
                        PatchOperation::SetData(data) => {
                            debug!("Patching data in node {}", patch.node_id);
                            node.node_mut().data_mut().data = data;
                        }
                    }
                } else {
                    error!("Node not found")
                }
            }
        })
    }
}
