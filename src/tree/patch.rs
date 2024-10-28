use std::hash::Hash;
use std::mem;

use arbutus::{TreeNode as _, TreeNodeRef as _};
use tracing::{debug, debug_span, error};

use crate::{
    attribute::Attributes,
    node::{SnowcapNode, SnowcapNodeData},
    IndexedTree, NodeId, NodeRef,
};

#[derive(Debug, Clone)]
pub enum PatchOperation<M>
where
    M: std::fmt::Debug + 'static,
{
    SetAttributes(Option<Attributes>),
    SetData(SnowcapNodeData),
    /*
    Update {
        attrs: Option<Attributes>,
        data: SnowcapNodeData,
        source: NodeRef<M>,
    },
    */
    AddChild { index: usize, node: SnowcapNode<M> },

    // Add a subtree at the specified child index of the parent node
    AddSubtree { index: usize, root: NodeRef<M> },

    Delete,
}

impl<M> Hash for PatchOperation<M>
where
    M: std::fmt::Debug + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

#[derive(Debug)]
pub struct TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    node_id: NodeId,
    operation: PatchOperation<M>,
}

impl<M> Hash for TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
        self.operation().hash(state)
    }
}

impl<M> Eq for TreePatch<M> where M: std::fmt::Debug + 'static {}

impl<M> PartialEq for TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)

        //let mut self_hasher = DefaultHasher::new();
        //self.hash(&mut self_hasher);
        //let self_hash = self.hash(&mut self_hasher).finish();
    }
}

impl<M> PartialOrd for TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.node_id.cmp(&other.node_id));

        /*
        match (self.operation(), other.operation()) {
            (
                PatchOperation::AddSubtree {
                    index: index_self, ..
                },
                PatchOperation::AddSubtree {
                    index: index_other, ..
                },
            ) => Some(index_self.cmp(index_other)),

            _ => {
                // In the default case, order based on the parent node ID,
                // putting early patches in the destination tree first
                Some(self.node_id.cmp(&other.node_id))
            }
        }
        */
    }
}

impl<M> Ord for TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.node_id.cmp(&other.node_id)
    }
}

#[allow(dead_code)]
impl<M> TreePatch<M>
where
    M: std::fmt::Debug + 'static,
{
    pub fn new(node_id: NodeId, operation: PatchOperation<M>) -> Self {
        Self { node_id, operation }
    }

    pub fn operation(&self) -> &PatchOperation<M> {
        &self.operation
    }
}

pub struct TreePatcher;

impl TreePatcher {
    pub fn patch<M>(tree: &mut IndexedTree<M>, patches: Vec<TreePatch<M>>)
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
                        PatchOperation::AddChild { index, node } => {
                            debug!("Adding new child to Node {} index {}", patch.node_id, index);

                            tree.insert_node(patch.node_id, index, node);
                        }
                        PatchOperation::Delete => {
                            if let Some(_removed) = tree.remove_node_id(&patch.node_id) {
                                debug!("Removed node {}", patch.node_id);
                            } else {
                                error!("Failed to remove node {}", patch.node_id);
                            }
                        }
                        PatchOperation::AddSubtree { index, root } => {
                            debug!("Adding subtree to node {}", patch.node_id);
                            let mut parent = node.clone();
                            tree.insert_subtree(&mut parent, index, root);
                        }
                    }
                } else {
                    error!("Node ID {} not found", patch.node_id)
                }

                tree.reindex();
            }
        })
    }
}
