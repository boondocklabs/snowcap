use std::{
    collections::{HashMap, HashSet, VecDeque},
    marker::PhantomData,
};

use arbutus::{TreeNode as _, TreeNodeRef as _};

use crate::{IndexedTree, NodeId, NodeRef};

pub struct TreeIter<M> {
    _phantom: PhantomData<M>,
}

impl<M> TreeIter<M> {
    /// Iterate through the tree backwards starting from the leaf nodes.
    pub fn backward<F>(tree: &IndexedTree<M>, mut f: F)
    where
        M: std::fmt::Debug,
        F: FnMut(&NodeRef<M>),
    {
        // Set of visited children for each parent node
        let mut children_visited: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

        // Get the list of leaf nodes from the tree
        let mut leaves = tree.leaves().clone();
        leaves.reverse();

        // Track which nodes have been visited, initialized with initial set of leaf nodes
        let mut visited: HashSet<NodeId> =
            leaves.iter().map(|leaf| leaf.node().id().clone()).collect();

        // Create the queue initialized with all leaf nodes of the tree
        let mut queue: VecDeque<NodeRef<M>> = VecDeque::from(leaves);

        // Queue of nodes at the next depth. This will move into the outer queue
        // after it has drained indicating all nodes at the current depth have
        // been processed.
        let mut next: VecDeque<NodeRef<M>> = VecDeque::new();

        while let Some(node) = queue.pop_front() {
            let node_id = node.node().id().clone();
            //info!("Pop {}", node_id);

            // Get the expected number of children for this node
            let expected_children = node.node().num_children();

            if expected_children > 0 {
                // Get the number of children we have visited for this node
                let have_children = children_visited.get(&node_id).map(|v| v.len()).unwrap_or(0);

                if expected_children != have_children {
                    // Put the node into the next depth queue, as not all children
                    // of this node have been resolved yet.

                    // Move next nodes into queue if the outer queue is empty
                    if queue.is_empty() {
                        queue.append(&mut next);
                    }

                    next.push_front(node);

                    // Continue the loop to pop the next node from the queue
                    continue;
                } else {
                    // All children have been resolved for the current node
                    // Remove the children hashset from children_visited
                    // We can only use this set if we don't need to traverse in a deterministic order
                    // For now, just ignore it
                    let _children = children_visited.remove(&node_id);

                    /*
                    if let Some(child_nodes) = node.node().children() {
                        for child in &*child_nodes {
                            //f(&child)
                        }
                    }
                    */

                    // Yield ourselves. All descendents of the children of this node have been previously yielded
                    // Note that it's up to the caller to iterate through the yielded node.node().children()
                    // in order to do what they please with the children
                    f(&node)
                }
            }

            // Check if this node has a parent
            if let Some(parent) = node.node().parent() {
                let parent_node = parent.node();
                let parent_id = parent_node.id();

                // Insert this node into children_visisted for the parent
                children_visited
                    .entry(parent_id)
                    .or_insert(HashSet::new())
                    .insert(node_id);

                if visited.insert(parent_id) {
                    // Node has not been visited before, add to queue
                    //info!("PUSH NEXT {}", parent_id);
                    next.push_front(parent.clone());
                }
            } else {
                // No parent. This is the root node.
                break;
            }

            if queue.is_empty() {
                queue.append(&mut next);
            }
        }
    }
}
