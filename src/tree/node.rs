use std::{
    cell::{Ref, RefMut},
    collections::VecDeque,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::atomic::AtomicU64,
};

use tracing::{debug, info, warn};

use crate::{
    conversion::widget::SnowcapWidget,
    message::{Event, WidgetMessage},
    MarkupTreeNode,
};

use super::hash::TreeNodeHasher;

pub type NodeId = u64;
pub type NodeHash = u64;

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(0);

pub struct TreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    id: NodeId,
    pub inner: Rc<MarkupTreeNode<'a, M>>,
    pub widget: Option<Box<SnowcapWidget<'a, M>>>,
}

impl<'a, M> std::fmt::Debug for TreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TreeNode(ID={})", self.id,))
    }
}

/// Clone the ID and the inner Rc
impl<'a, M> Clone for TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn clone(&self) -> TreeNode<'a, M> {
        debug!("TreeNode {self:#?} CLONE");

        TreeNode {
            id: self.id,
            inner: self.inner.clone(),
            widget: None,
        }
    }
}

impl<'a, M> TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn new(inner: MarkupTreeNode<'a, M>) -> Self
    where
        M: From<WidgetMessage>,
    {
        let id = NEXT_NODE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        info!("Created TreeNode ID {id} for {inner:#?}");

        Self {
            id,
            inner: Rc::new(inner),
            widget: None,
        }
    }

    pub fn borrow<'b>(&'b self) -> NodeRef<'a, 'b, M> {
        NodeRef::new(self)
    }

    /// Get the xx64 hash of this node
    pub fn greedy_hash(&self) -> NodeHash
    where
        M: From<Event> + From<WidgetMessage>,
    {
        TreeNodeHasher::Greedy.hash(&self)
    }

    pub fn thrify_hash(&self) -> NodeHash
    where
        M: From<Event> + From<WidgetMessage>,
    {
        TreeNodeHasher::Thrifty.hash(&self)
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn set_widget(&mut self, widget: SnowcapWidget<'a, M>) {
        self.widget = Some(Box::new(widget))
    }

    pub fn take_widget(&mut self) -> Option<Box<SnowcapWidget<'a, M>>> {
        self.widget.take()
    }

    pub fn get_widget(&'a self) -> Option<&'a SnowcapWidget<'a, M>> {
        if let Some(widget) = &self.widget {
            Some(&*widget)
        } else {
            None
        }
    }
}

/// Implement dereference on a TreeNode, providing a reference to the inner MarkupTreeNode
impl<'a, M> Deref for TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Target = MarkupTreeNode<'a, M>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

/// A mutable reference to a tree node
pub struct NodeRefMut<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    node_ref: RefMut<'a, MarkupTreeNode<'a, M>>,
}

impl<'a, M, T> AsRef<T> for NodeRefMut<'a, M>
where
    T: ?Sized,
    <NodeRefMut<'a, M> as Deref>::Target: AsRef<T>,
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<'a, M> Deref for NodeRefMut<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Target = MarkupTreeNode<'a, M>;

    fn deref(&self) -> &Self::Target {
        &*(self.node_ref)
    }
}

impl<'a, M> DerefMut for NodeRefMut<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *(self.node_ref)
    }
}

/// A reference to a tree node
#[derive(Debug, Clone)]
pub struct NodeRef<'t, 'r, M>
where
    't: 'r,
    M: std::fmt::Debug + From<WidgetMessage> + 't,
{
    node_ref: Rc<&'r TreeNode<'t, M>>,
}

impl<'t, 'r, M> NodeRef<'t, 'r, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 't,
{
    pub fn new(node: &'r TreeNode<'t, M>) -> Self {
        Self {
            node_ref: Rc::new(node),
        }
    }

    pub fn inner<'x>(&'x self) -> &'r MarkupTreeNode<'t, M> {
        &self.node_ref.inner
    }
}

impl<'a, 'b, M> Deref for NodeRef<'a, 'b, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Target = TreeNode<'a, M>;

    fn deref(&self) -> &Self::Target {
        *(self.node_ref)
    }
}

impl<'a, M> From<MarkupTreeNode<'a, M>> for TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn from(value: MarkupTreeNode<'a, M>) -> Self {
        TreeNode::new(value)
    }
}

impl<'r, 't, M> IntoIterator for &'r TreeNode<'t, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 't,
{
    type Item = NodeRef<'t, 'r, M>;
    type IntoIter = TreeNodeIterRef<'t, 'r, M>;

    fn into_iter(self) -> Self::IntoIter {
        // Create an iterator starting with the root node in the stack
        TreeNodeIterRef {
            stack: VecDeque::from([NodeRef::new(self)]),
        }
    }
}

pub struct TreeNodeIterRef<'t, 'r, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 't,
{
    stack: VecDeque<NodeRef<'t, 'r, M>>,
}

impl<'t, 'r, M> Iterator for TreeNodeIterRef<'t, 'r, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 't,
    't: 'r,
{
    type Item = NodeRef<'t, 'r, M>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        current.map(|node| {
            match node.inner() {
                MarkupTreeNode::Container { content, .. }
                | MarkupTreeNode::Widget { content, .. } => {
                    self.stack.push_front(content.borrow());
                }
                MarkupTreeNode::Row { contents, .. }
                | MarkupTreeNode::Column { contents, .. }
                | MarkupTreeNode::Stack { contents, .. } => {
                    for content in contents.iter().rev() {
                        self.stack.push_front(content.borrow());
                    }
                }
                _ => {}
            }
            node.clone()
        })
    }
}

impl<'a, M: 'a> IntoIterator for TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Item = TreeNode<'a, M>;
    type IntoIter = TreeNodeIter<'a, M>;

    fn into_iter(self) -> Self::IntoIter {
        TreeNodeIter {
            stack: VecDeque::from([self]),
        }
    }
}

pub struct TreeNodeIter<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    stack: VecDeque<TreeNode<'a, M>>,
}

impl<'a, M> Iterator for TreeNodeIter<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Item = TreeNode<'a, M>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        if let Some(node) = current {
            match &*node.inner {
                MarkupTreeNode::Container { content, .. }
                | MarkupTreeNode::Widget { content, .. } => {
                    self.stack.push_front(content.clone());
                }
                MarkupTreeNode::Row { contents, .. }
                | MarkupTreeNode::Column { contents, .. }
                | MarkupTreeNode::Stack { contents, .. } => {
                    for content in contents.iter().rev() {
                        self.stack.push_front(content.clone());
                    }
                }
                _ => {}
            }
            Some(node)
        } else {
            None
        }
    }
}

impl<'a, AppMessage> std::hash::Hash for TreeNode<'a, AppMessage>
where
    AppMessage: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use tracing::info;
    use tracing_test::traced_test;

    use crate::message::WidgetMessage;
    use crate::node_manager::NodeManager;
    use crate::{attribute::Attributes, Message};

    use super::{MarkupTreeNode, NodeRef};
    use super::{NodeId, TreeNode};

    type M = Message<String>;

    fn row_tree<'a>() -> TreeNode<'a, M> {
        let a = TreeNode::new(MarkupTreeNode::Label("Hello".into()));
        let b = TreeNode::new(MarkupTreeNode::Label("World".into()));

        let row = MarkupTreeNode::Row {
            element_id: None,
            attrs: Attributes::default(),
            contents: Vec::from([a, b]),
        };

        let root = TreeNode::new(row);
        root
    }

    #[traced_test]
    #[test]
    fn test_iterator() {
        let tree = row_tree();

        for node in &tree {
            info!("Iter node {node:#?}");
        }
    }

    #[derive(Debug)]
    struct Index<'a, M>
    where
        M: std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        pub index: BTreeMap<NodeId, NodeRef<'a, 'a, M>>,
    }

    #[traced_test]
    #[test]
    fn test_reference() {
        let mut index = Index::<'static, M> {
            index: BTreeMap::new(),
        };

        let tree = row_tree();
        let r = tree.borrow();

        index.index.insert(r.id, r);

        tree.get_element_id();
    }
}
