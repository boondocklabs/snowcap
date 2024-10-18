use iced::advanced::Widget;

type Node<Data, Id> = arbutus::TreeNodeSimple<Data, Id>;
type NodeRef<M> = arbutus::NodeRefRef<Node<CacheTreeNode<M>, arbutus::NodeId>>;

type CacheTree<M> = arbutus::Tree<NodeRef<M>>;
type CacheIndexedTree<M> = arbutus::IndexedTree<NodeRef<M>>;
type NodeId = arbutus::NodeId;

struct CacheTreeNode<M> {
    widget: Box<dyn Widget<M, iced::Theme, iced::Renderer>>,
}

impl<M> std::fmt::Display for CacheTreeNode<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheTreeNode")
    }
}

// Widget cache is an indexed tree of Box<dyn Widget>
pub struct WidgetCache<M>
where
    M: 'static,
{
    tree: CacheIndexedTree<M>,
}

impl<M> WidgetCache<M> {
    pub fn new() -> Self {
        Self {
            tree: CacheIndexedTree::new(),
        }
    }
}
