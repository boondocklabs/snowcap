use arbutus::{TreeNode, TreeNodeRef as _};
use iced::Element;
use tracing::{debug, debug_span};

use crate::{
    attribute::Attributes,
    conversion::{
        column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, stack::SnowcapStack,
        widget::SnowcapWidget,
    },
    dynamic_widget::DynamicWidget,
    message::WidgetMessage,
    node::SnowcapNodeData,
    ConversionError, IndexedTree, NodeId, Value,
};

/// Widget content passed to the widget builders to provide their content.
#[derive(Debug)]
pub enum WidgetContent<M> {
    None,
    Widget(DynamicWidget<M>),
    Value(Value),
    List(Vec<Self>),
}

impl<M> std::fmt::Display for WidgetContent<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WidgetContent::None => write!(f, "None"),
            WidgetContent::Widget(dynamic_widget) => write!(f, "{:?}", dynamic_widget),
            WidgetContent::Value(value) => write!(f, "{}", value),
            WidgetContent::List(vec) => {
                write!(f, "[")?;
                for w in vec {
                    write!(f, "{:?},", w)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl<M> Into<Element<'static, M>> for WidgetContent<M>
where
    M: 'static,
{
    fn into(self) -> Element<'static, M> {
        match self {
            WidgetContent::Widget(dynamic_widget) => dynamic_widget.into_element().unwrap(),
            _ => unimplemented!("Cannot convert non-Widget type content into an Element"),
        }
    }
}

impl<M> IntoIterator for WidgetContent<M>
where
    M: std::fmt::Debug + 'static,
{
    type Item = Element<'static, M>;

    type IntoIter = std::vec::IntoIter<Element<'static, M>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            WidgetContent::Widget(widget) => vec![widget.into_element().unwrap()].into_iter(),
            WidgetContent::List(vec) => vec
                .into_iter()
                .filter_map(|item| {
                    if let WidgetContent::Widget(widget) = item {
                        Some(widget.into_element().unwrap())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .into_iter(),
            _ => {
                tracing::warn!("Attempt to iterate non iterable type {:?}", self);
                Vec::new().into_iter()
            } // Return an empty iterator for non-iterable types
        }
    }
}

#[derive(Debug)]
pub struct WidgetCache;

impl WidgetCache {
    pub fn build_widgets<M>(tree: &IndexedTree<M>) -> Result<(), ConversionError>
    where
        M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        debug_span!("build-widgets-mark").in_scope(|| {
            // First pass, mark dirty nodes and drop widgets. We do this as a first pass
            // in a different scope so the RwLock write guards in WidgetRef are released.
            // The parent of a node holds its WidgetRef in the contents of the parent,
            // so we drop all widgets along the path before rebuilding. Any nodes not affected
            // by a dirty path will be reused.
            tree.leaf_iter().for_each(|noderef| {
                let mut node = noderef.node_mut();

                if node.data().dirty == true {
                    // Drop the old widget
                    debug!("Drop dirty widget from node {:?}", node.id());
                    drop(node.data_mut().widget.take());

                    // Mark the parent widget as dirty
                    if let Some(parent) = node.parent_mut() {
                        parent.node_mut().data_mut().dirty = true;
                    }

                    // Clear the dirty flag
                    node.data_mut().dirty = false;
                }

                Ok::<(), ConversionError>(())
            })
        })?;

        debug_span!("build-widgets").in_scope(|| {
            tree.leaf_iter().for_each(|noderef| {
                let _node_id = noderef.node().id();

                let mut node = noderef.node_mut();
                let data = node.data();
                let node_id = node.id();
                let attrs = data.attrs.clone().unwrap_or(Attributes::default());

                if data.widget.is_some() {
                    debug!("Reusing widget {node_id}");
                    // Already have a widget
                    return Ok(());
                }

                let child_widgets: Option<Vec<DynamicWidget<M>>> =
                    node.children().and_then(|children| {
                        let widgets: Vec<DynamicWidget<M>> = children
                            .iter()
                            .filter_map(|child| child.node().data().widget.clone())
                            .collect();

                        (!widgets.is_empty()).then_some(widgets)
                    });

                let content = if let Some(mut children) = child_widgets {
                    if children.len() == 1 {
                        WidgetContent::Widget(children.pop().unwrap())
                    } else {
                        WidgetContent::List(
                            children
                                .into_iter()
                                .map(|w| WidgetContent::Widget(w))
                                .collect(),
                        )
                    }
                } else {
                    if node.num_children() == 1 {
                        let child = node.children().unwrap().iter().last().unwrap();

                        if let SnowcapNodeData::Value(value) = &child.node().data().data {
                            WidgetContent::Value(value.clone())
                        } else {
                            WidgetContent::None
                        }
                    } else {
                        WidgetContent::None
                    }
                };

                let widget = match &**data {
                    SnowcapNodeData::Widget(widget) => {
                        debug!("Building widget {widget} node {node_id} contents {content}");

                        let widget = SnowcapWidget::new(
                            node_id,
                            widget.clone(),
                            data.element_id.clone(),
                            attrs,
                            content,
                        )?
                        .with_node_id(node_id);

                        Some(widget)
                    }
                    SnowcapNodeData::Container => {
                        debug!("Building Container node {node_id} contents {content}");
                        let widget = SnowcapContainer::new(attrs, content)?.with_node_id(node_id);
                        Some(widget)
                    }
                    SnowcapNodeData::Row => {
                        debug!("Building Row node {node_id} contents {content}");
                        let widget = SnowcapRow::convert(attrs, content)?.with_node_id(node_id);
                        Some(widget)
                    }
                    SnowcapNodeData::Column => {
                        debug!("Building Column node {node_id} contents {content}");
                        let widget = SnowcapColumn::convert(attrs, content)?.with_node_id(node_id);
                        Some(widget)
                    }
                    SnowcapNodeData::Stack => {
                        debug!("Building Stack node {node_id} contents {content}");
                        let widget = SnowcapStack::convert(attrs, content)?.with_node_id(node_id);
                        Some(widget)
                    }
                    SnowcapNodeData::Root => {
                        debug!("Building Root node {node_id} contents {content}");
                        if let WidgetContent::Widget(widget) = content {
                            Some(widget)
                        } else {
                            panic!("No widget in root");
                        }
                    }
                    SnowcapNodeData::Value(_) => None,
                    SnowcapNodeData::None => None,
                };

                if let Some(widget) = widget {
                    let old = node.data_mut().widget.replace(widget);
                    drop(old);
                }

                Ok::<(), ConversionError>(())
            })?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{tree_util::WidgetCache, Message, SnowcapParser};

    #[traced_test]
    #[test]
    pub fn build_tree_widgets() {
        let tree = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("A"), text("B"), text("C")]}"#,
        )
        .unwrap()
        .index();

        println!("{}", tree.root());

        WidgetCache::build_widgets(&tree);
    }
}
