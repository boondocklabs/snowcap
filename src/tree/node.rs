use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    ops::{Deref, DerefMut},
    process::exit,
    rc::Rc,
    sync::atomic::AtomicU64,
};

use tracing::info;

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
    pub inner: Rc<RefCell<MarkupTreeNode<'a, M>>>,
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

/// Clone the ID and the inner Arc
impl<'a, M> Clone for TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn clone(&self) -> TreeNode<'a, M> {
        info!("TreeNode {self:#?} CLONE");

        // Rebuild the widget for this node
        let tree_node = self.inner.borrow().clone();
        let widget = SnowcapWidget::from_node(tree_node)
            .unwrap()
            .map(|w| Box::new(w));

        TreeNode {
            id: self.id,
            inner: self.inner.clone(),
            widget,
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
        let widget = SnowcapWidget::from_node(inner.clone()).unwrap();

        let id = NEXT_NODE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        info!("\n\n\n!!!!!WIDGET BUILT FOR NODE {id}!!!!!!\n\n\n");
        info!("Created TreeNode ID {id} for {inner:#?}");

        Self {
            id,
            inner: Rc::new(RefCell::new(inner)),
            widget: widget.map(|w| Box::new(w)),
        }
    }

    /// Get the xx64 hash of this node
    pub fn greedy_hash(&self) -> NodeHash
    where
        M: From<Event> + From<WidgetMessage>,
    {
        TreeNodeHasher::Greedy.hash(self.clone())
    }

    pub fn thrify_hash(&self) -> NodeHash
    where
        M: From<Event> + From<WidgetMessage>,
    {
        TreeNodeHasher::Thrifty.hash(self.clone())
    }

    pub fn inner(&'a self) -> NodeRef<'a, M> {
        let node = (*self.inner).borrow();
        NodeRef { node_ref: node }
    }

    pub fn inner_ref(&self) -> Ref<'_, MarkupTreeNode<'a, M>> {
        self.inner.borrow()
    }

    pub fn inner_mut(&'a mut self) -> NodeRefMut<'a, M> {
        NodeRefMut {
            node_ref: (*self.inner).borrow_mut(),
        }
    }

    pub fn replace_inner(&self, inner: MarkupTreeNode<'a, M>) {
        *self.inner.deref().borrow_mut() = inner;
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
pub struct NodeRef<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    node_ref: Ref<'a, MarkupTreeNode<'a, M>>,
}

impl<'a, M> NodeRef<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn as_ref(&'a self) -> &'a MarkupTreeNode<M> {
        &*self.node_ref
    }
}

impl<'a, M, T> AsRef<T> for NodeRef<'a, M>
where
    T: ?Sized,
    <NodeRef<'a, M> as Deref>::Target: AsRef<T>,
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl<'a, M> Deref for NodeRef<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Target = MarkupTreeNode<'a, M>;

    fn deref(&self) -> &Self::Target {
        &*(self.node_ref)
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

impl<'a, M> TreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    // Define an `into_iter` method that returns an iterator
    // which yields a reference to each node in the tree
    pub fn iter(&self) -> TreeNodeIter<'a, M>
    where
        M: Clone + std::fmt::Debug + 'a,
    {
        TreeNodeIter {
            stack: VecDeque::from([self.clone()]),
        }
    }
}

impl<'a, M: 'a> IntoIterator for &TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Item = TreeNode<'a, M>;
    type IntoIter = TreeNodeIter<'a, M>;

    fn into_iter(self) -> Self::IntoIter {
        TreeNodeIter {
            stack: VecDeque::from([self.clone()]),
        }
    }
}

impl<'a, M> IntoIterator for &mut TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    type Item = TreeNode<'a, M>;
    type IntoIter = TreeNodeIter<'a, M>;

    fn into_iter(self) -> Self::IntoIter {
        TreeNodeIter {
            stack: VecDeque::from([self.clone()]),
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

        if let Some(tree) = current {
            match &*tree.inner_ref() {
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
            Some(tree)
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
        self.inner.borrow().hash(state);
    }
}
