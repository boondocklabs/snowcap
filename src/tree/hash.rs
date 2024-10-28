use arbutus::{NodeBuilder, NodePosition, Tree, TreeBuilder, TreeNode, TreeNodeRef};

/*
pub(crate) type HashNode<Data, Id> = arbutus::node::refcell::Node<Data, Id>;
pub(crate) type HashNodeRef = <HashNode<TreeHashNode, arbutus::NodeId> as TreeNode>::NodeRef;

type HashNodeBuilder<'a> = NodeBuilder<
    'a,
    TreeHashNode,
    (),
    arbutus::IdGenerator,
    HashNode<TreeHashNode, arbutus::NodeId>,
    HashNodeRef,
>;

use crate::IndexedTree;

#[derive(Clone, Hash, Debug, Default)]
pub struct TreeHashNode {
    hash: Option<NodeHash>,
}

impl TreeHashNode {
    pub fn nodehash(&self) -> Option<&NodeHash> {
        self.hash.as_ref()
    }
}

impl std::fmt::Display for TreeHashNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(nodehash) = self.nodehash() {
            return write!(f, "NodeHash [0x{:X}]", nodehash.hash_value());
        }

        panic!("NodeHash not available");
    }
}

impl<T: TreeNodeRef> From<(NodePosition, T)> for TreeHashNode {
    fn from(value: (NodePosition, T)) -> Self {
        Self {
            hash: Some(NodeHash::from(value)),
        }
    }
}

#[derive(Debug)]
pub struct TreeHash {
    root: HashNodeRef,
}

/// A tree of NodeHash values which shadows each node and structure of a Snowcap Tree
impl TreeHash {
    pub fn update(tree: &mut arbutus::IndexedTree<HashNodeRef>) {
        for mut node in tree.root().into_iter() {
            let pos = *node.position();
            let mut inner = node.node_mut();
            let mut data = inner.data_mut();

            if let Some(nodehash) = &mut data.hash {
                if *nodehash.position() != pos {
                    tracing::info!(
                        "Updating position from {:?} to {:?}",
                        nodehash.position(),
                        pos
                    );
                    nodehash.set_position(pos);
                }
            }
        }
    }

    fn add_children<M>(nb: &mut HashNodeBuilder, node: &crate::NodeRef<M>) {
        if let Some(children) = node.node().children() {
            for child in &*children {
                nb.child(TreeHashNode::default(), |nb| {
                    // Create a NodeHash from the tree position and SnowcapNodeData hash
                    nb.node_mut().node_mut().data_mut().hash =
                        Some(NodeHash::from((nb.position().clone(), (*child).clone())));

                    Self::add_children(nb, child);
                    Ok(())
                })
                .unwrap();
            }
        }
    }

    pub fn from_tree<M>(tree: &IndexedTree<M>) -> Option<Tree<HashNodeRef>> {
        let mut iter = tree.root().into_iter();
        let root = &*iter.next()?;

        let root_node = TreeHashNode::from((
            NodePosition {
                depth: 0,
                index: 0,
                child_index: 0,
            },
            root.clone(),
        ));

        let tb = TreeBuilder::<TreeHashNode, ()>::new();

        let hashtree = tb
            .root(root_node, |nb| {
                Self::add_children(nb, root);
                Ok(())
            })
            .unwrap()
            .done()
            .unwrap()
            .unwrap();

        Some(hashtree)
    }
}
*/

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::{Message, SnowcapParser};

    #[traced_test]
    #[test]
    fn tree_hash() {
        let a = SnowcapParser::<Message<String>>::parse_memory(
            r#"
        {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
            // Left column
            col[
                //text("inserting"),
                svg(file!("samples/coder.svg")),
                pick-list#foo<selected:"abc">(["abc", "bar"]),
                pick-list#bar<selected:"bar">(["baz", "bar"]),
                text<size:24>("I'm some text"),
                text<size:10>("More text in a Column"),
                text(url!("http://icanhazip.com")),
                image(url!("https://picsum.photos/200/300")),
                space<size:10>(),
                {<height:fill, align-y:center>
                    text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                }
            ]
        }
        "#,
        ).unwrap().index();

        let th = TreeHash::from_tree::<Message<String>>(&a).unwrap();
        println!("{th:#?}");

        println!("{}", a.root());
        println!("{}", th.root());

        // Check that the positions in the hash structs match the positions from the tree iterator

        for inode in th.root() {
            let node = inode.node();
            let data = node.data();

            assert_eq!(*inode.position(), *data.hash.as_ref().unwrap().position());
        }
    }
}
