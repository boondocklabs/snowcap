use tracing::{debug, debug_span};

use crate::{
    message::{Event, WidgetMessage},
    tree::hash::TreeNodeHasher,
    MarkupTreeNode,
};

use super::node::TreeNode;

pub struct TreeDiff;

impl TreeDiff {
    pub fn diff<'a, M>(input_current: TreeNode<'a, M>, input_new: TreeNode<'a, M>)
    where
        M: Clone + std::fmt::Debug + From<Event> + From<WidgetMessage> + 'a,
    {
        let span = debug_span!("diff");
        let _enter = span.enter();

        debug!(
            r#"
Comparing Trees

Current:
{:#?}

New:
{:#?}

"#,
            input_current, input_new
        );

        let current_greedy = TreeNodeHasher::Greedy.hash(&input_current);
        let new_greedy = TreeNodeHasher::Greedy.hash(&input_new);

        if current_greedy == new_greedy {
            debug!("Greedy hashes of both trees are equal. Trees are identical.");
            return;
        }

        let mut current_iter = input_current.iter();
        let mut new_iter = input_new.iter();

        let mut parent: Option<TreeNode<M>> = None;

        /*
        loop {
            let current_node = current_iter.next();
            let new_node = new_iter.next();

            if let (Some(current), Some(new)) = (current_node.clone(), new_node) {
                if current.thrify_hash() != new.thrify_hash() {
                    debug!("Nodes differ between\n\n{current:?}\n\nand\n\n{new:?}");

                    let new_inner: MarkupTreeNode<M> = new.inner.clone();

                    current.replace_inner(new_inner);

                    debug!("NEW {input_current:#?}");
                    continue;
                }
            } else {
                debug!("Current tree iter complete");

                while let Some(new) = new_iter.next() {
                    debug!("Additional new node {new:#?}");

                    debug!("Parent {parent:#?}");
                }

                break;
            }

            parent = current_node;
        }
        */

        debug!("CHECK AFTER PATCH");
        //Self::diff(input_current, input_new);
    }
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use crate::{tree::diff::TreeDiff, Message, SnowcapParser};

    #[traced_test]
    #[test]
    fn identical() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#).unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#).unwrap();

        TreeDiff::diff(a, b);
    }

    #[traced_test]
    #[test]
    fn different() {
        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#).unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#).unwrap();
        TreeDiff::diff(a, b);

        let a = SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("Hello")]}"#).unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#).unwrap();
        TreeDiff::diff(a, b);

        let a = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("Hello"), col[text("Test")]]}"#,
        )
        .unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#).unwrap();
        TreeDiff::diff(a, b);

        let a =
            SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("A"), text("B")]}"#).unwrap();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("A"), text("B"), text("C")]}"#,
        )
        .unwrap();
        TreeDiff::diff(a, b);
    }
}
