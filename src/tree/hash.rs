use std::hash::{Hash, Hasher};

use tracing::{debug_span, warn};
use xxhash_rust::xxh64::Xxh64;

use crate::{
    attribute::{Attribute, Attributes},
    data::DataType,
    message::{Event, WidgetMessage},
    MarkupTreeNode, Value,
};

use super::node::{NodeHash, TreeNode};

impl Hash for DataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DataType::Null => {}
            DataType::Image(handle) => handle.id().hash(state),
            DataType::Svg(handle) => handle.id().hash(state),
            DataType::QrCode(_data) => {
                warn!("Can't hash QRCode");
            }
            DataType::Markdown(_markdown_items) => {
                warn!("Can't hash Markdown");
                /*
                for item in &**markdown_items.inner() {
                }
                */
            }
            DataType::Text(text) => text.hash(state),
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::String(s) => s.hash(state),
            Value::Number(num) => state.write(&num.to_ne_bytes()),
            Value::Boolean(b) => b.hash(state),
            Value::Array(vec) => vec.hash(state),
            Value::Data { data, provider: _ } => {
                data.hash(state);
                // TODO: Hash provider state
            }
        }
    }
}

impl Hash for Attribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.value().hash(state);
    }
}

impl Hash for Attributes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for attr in self {
            attr.hash(state);
        }
    }
}

impl<'a, M> Hash for MarkupTreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MarkupTreeNode::None => {}
            MarkupTreeNode::Container {
                element_id,
                attrs,
                content: _,
                ..
            } => {
                element_id.hash(state);
                attrs.hash(state);
                //content.inner().hash(state);
            }
            MarkupTreeNode::Widget {
                element_id,
                name,
                attrs,
                content: _,
            } => {
                element_id.hash(state);
                name.hash(state);
                attrs.hash(state);
                //content.inner().hash(state);
            }
            MarkupTreeNode::Row {
                element_id,
                attrs,
                contents: _,
            } => {
                element_id.hash(state);
                attrs.hash(state);
                /*
                for content in &**contents {
                    content.inner().hash(state);
                }
                */
            }
            MarkupTreeNode::Column {
                element_id,
                attrs,
                contents: _,
            } => {
                element_id.hash(state);
                attrs.hash(state);
                /*
                for content in &**contents {
                    content.inner().hash(state);
                }
                */
            }
            MarkupTreeNode::Stack {
                element_id,
                attrs,
                contents: _,
            } => {
                element_id.hash(state);
                attrs.hash(state);
                /*
                for content in &**contents {
                    content.inner().hash(state);
                }
                */
            }
            MarkupTreeNode::Label(label) => label.hash(state),
            MarkupTreeNode::Value(val) => (**val).borrow().hash(state),
            MarkupTreeNode::Phantom(phantom_data) => phantom_data.hash(state),
        }
    }
}

#[derive(Debug)]
pub enum TreeNodeHasher {
    Thrifty,
    Greedy,
}

impl TreeNodeHasher {
    fn hash_with<'a, M>(&'a self, node: TreeNode<'a, M>, hasher: &mut impl Hasher)
    where
        M: Clone + std::fmt::Debug + From<Event> + From<WidgetMessage> + 'a,
    {
        match self {
            TreeNodeHasher::Thrifty => {
                // Hash this node without any children
                node.hash(hasher);
            }
            TreeNodeHasher::Greedy => {
                // Hash this node and all children
                node.hash(hasher);

                // Recurse into child nodes
                let mut iter = node.iter();
                iter.next();
                for child in iter {
                    self.hash_with(child, hasher);
                }
            }
        }
    }

    pub fn hash<'a, M>(&'a self, tree: TreeNode<'a, M>) -> NodeHash
    where
        M: Clone + std::fmt::Debug + From<Event> + From<WidgetMessage> + 'a,
    {
        debug_span!("hasher").in_scope(|| {
            let mut hasher = Xxh64::new(0);
            self.hash_with(tree, &mut hasher);
            hasher.finish()
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, SnowcapParser};
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_hash_markup() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#).unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#).unwrap();
        let c = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#).unwrap();

        let hash_a = TreeNodeHasher::Greedy.hash(a);
        let hash_b = TreeNodeHasher::Greedy.hash(b);
        let hash_c = TreeNodeHasher::Greedy.hash(c);

        assert_eq!(hash_a, hash_b, "Hashes of the same markup should match");
        assert_ne!(
            hash_a, hash_c,
            "Hashes of different string in text content should have different hash"
        );
    }
}
