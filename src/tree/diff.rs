use std::collections::{HashMap, HashSet};

use crate::{message::Event, tree::patch::PatchOperation, IndexedTree, NodeId};
use arbutus::{Node, NodeRef};
use tracing::info;

use super::patch::TreePatch;

pub struct TreeDiff;

impl TreeDiff {
    pub fn diff<'a, M>(a: &IndexedTree<M>, b: &IndexedTree<M>) -> Vec<TreePatch>
    where
        M: Clone + std::fmt::Debug + From<Event> + 'a,
    {
        info!("Diffing");

        let inverted_index_a: HashMap<u64, NodeId> = a
            .root()
            .into_iter()
            .map(|node| (node.node().data().xxhash(), node.node().id().clone()))
            .collect();

        let inverted_index_b: HashMap<u64, NodeId> = b
            .root()
            .into_iter()
            .map(|node| (node.node().data().xxhash(), node.node().id().clone()))
            .collect();

        let set_a: HashSet<u64> = inverted_index_a
            .keys()
            .map(|node_hash| node_hash.clone())
            .collect();

        let set_b: HashSet<u64> = inverted_index_b
            .keys()
            .map(|node_hash| node_hash.clone())
            .collect();

        //info!("SetA:\n{set_a:#?}");
        //info!("SetB:\n{set_b:#?}");

        // Nodes in A which don't exist in B
        let mut diff_a: HashSet<NodeId> = set_a
            .difference(&set_b)
            .map(|h| inverted_index_a[h])
            .collect();

        // Nodes in B which don't exist in A
        let mut diff_b: HashSet<NodeId> = set_b
            .difference(&set_a)
            .map(|h| inverted_index_b[h])
            .collect();

        info!("Diff A\n{diff_a:#?}");
        info!("Diff B:\n{diff_b:#?}");

        let mut patches = Vec::<TreePatch>::new();

        for id in &diff_a {
            let node = a.get_node(id);
            info!("{node:#?}");
        }

        // If we have exactly one different node in each tree, issue an op to update the node
        if diff_a.len() == 1 && diff_b.len() == 1 {
            let id_a = diff_a.drain().last().unwrap();
            let id_b = diff_b.drain().last().unwrap();

            let node_a = a.get_node(&id_a).unwrap();
            let node_b = b.get_node(&id_b).unwrap();

            let cmp = node_a.node().data().compare(&node_b.node().data());
            let _patch = match cmp {
                crate::tree::compare::SnowcapNodeComparison::Equal => {
                    panic!("Nodes should not be equal")
                }
                crate::tree::compare::SnowcapNodeComparison::DataDiffer => {
                    patches.push(TreePatch::new(
                        id_a,
                        PatchOperation::SetData(node_b.node().data().data.clone()),
                    ));
                }
                crate::tree::compare::SnowcapNodeComparison::AttributeDiffer => {
                    patches.push(TreePatch::new(
                        id_a,
                        PatchOperation::SetAttributes(node_b.node().data().attrs.clone()),
                    ));
                }
                crate::tree::compare::SnowcapNodeComparison::BothDiffer => {
                    patches.push(TreePatch::new(
                        id_a,
                        PatchOperation::SetAttributes(node_b.node().data().attrs.clone()),
                    ));
                    patches.push(TreePatch::new(
                        id_a,
                        PatchOperation::SetData(node_b.node().data().data.clone()),
                    ));
                }
            };
        }

        info!("PATCHES {patches:#?}");

        /*
        for diff in set_a.difference(&set_b) {
            info!("DIFF HASH {}", diff);
            let node_a = a.get_node(&inverted_index_a[diff]).unwrap();
            info!("DIFF NODE {node_a:?}");

            if let Some(parent) = node_a.node().parent() {
                let parent_hash = parent.node().data().xxhash();
                info!("Parent Hash {parent_hash}");

                // Find parent ID in Tree B inverted index
                let parent_id_b = inverted_index_b.get(&parent_hash);

                if let Some(parent_id_b) = parent_id_b {
                    let parent_b = b.get_node(parent_id_b).unwrap();
                    info!("WE GOT PARENT NODE FROM TREE B: {parent_b}")
                } else {
                    info!("Parent hash not found in inverted index for Tree B");
                }
            }
        }
        */

        patches
    }
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use crate::{tree::diff::TreeDiff, Message, SnowcapParser};

    #[traced_test]
    #[test]
    fn identical() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();

        TreeDiff::diff(&a, &b);
    }

    #[traced_test]
    #[test]
    fn different() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        TreeDiff::diff(&a, &b);

        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("Hello")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        TreeDiff::diff(&a, &b);

        let a = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("Hello"), col[text("Test")]]}"#,
        )
        .unwrap()
        .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        TreeDiff::diff(&a, &b);

        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("A"), text("B")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("A"), text("B"), text("C")]}"#,
        )
        .unwrap()
        .index();
        TreeDiff::diff(&a, &b);
    }

    #[traced_test]
    #[test]
    fn different_attrs() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("A")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:20>("A")}"#)
            .unwrap()
            .index();
        let patches = TreeDiff::diff(&a, &b);

        assert_eq!(patches.len(), 1);
    }

    #[traced_test]
    #[test]
    fn different_text() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("A")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("B")}"#)
            .unwrap()
            .index();
        let patches = TreeDiff::diff(&a, &b);

        assert_eq!(patches.len(), 1);

        let patch = patches.last().unwrap();
        match patch.operation() {
            crate::tree::diff::PatchOperation::SetAttributes(_attributes) => {
                panic!("Unexpected attribute patch")
            }
            crate::tree::diff::PatchOperation::SetData(_snowcap_node_data) => {}
        }
    }
}
