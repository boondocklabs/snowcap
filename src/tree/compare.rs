use crate::node::SnowcapNode;

#[derive(Debug, PartialEq, Eq)]
pub enum SnowcapNodeComparison {
    Equal,
    DataDiffer,
    AttributeDiffer,
    BothDiffer,
}

impl<M> SnowcapNode<M> {
    /// Compare two SnowcapNode instances, returning a SnowcapNodeComparison enum
    /// describing if the nodes are the same or if the data, attributes, or both are different
    pub fn compare(&self, other: &Self) -> SnowcapNodeComparison {
        let data_equal = self.data.xxhash() == other.data.xxhash();

        let attrs_equal = self
            .attrs
            .as_ref()
            .zip(other.attrs.as_ref()) // Combine the two Options if both are Some
            .map_or(
                self.attrs.is_none() && other.attrs.is_none(),
                |(ours, theirs)| ours.xxhash() == theirs.xxhash(),
            );

        if data_equal && attrs_equal {
            SnowcapNodeComparison::Equal
        } else if data_equal && !attrs_equal {
            SnowcapNodeComparison::AttributeDiffer
        } else if !data_equal && attrs_equal {
            SnowcapNodeComparison::DataDiffer
        } else {
            SnowcapNodeComparison::BothDiffer
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{attribute::Attribute, node::SnowcapNodeData, Message};

    use super::*;
    use crate::attribute::Attributes;
    use tracing_test::traced_test;

    type M = Message<String>;

    #[traced_test]
    #[test]
    fn test_equal() {
        let a = SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into()));
        let b = SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into()));
        assert_eq!(a.compare(&b), SnowcapNodeComparison::Equal);

        let mut attr_a = Attributes::new();
        attr_a
            .push(Attribute::new(crate::attribute::AttributeValue::Label(
                "foo".into(),
            )))
            .ok();

        let mut attr_b = Attributes::new();
        attr_b
            .push(Attribute::new(crate::attribute::AttributeValue::Label(
                "foo".into(),
            )))
            .ok();

        let a =
            SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(Some(attr_a));
        let b =
            SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(Some(attr_b));
        assert_eq!(a.compare(&b), SnowcapNodeComparison::Equal);
    }

    #[traced_test]
    #[test]
    fn test_data_differ() {
        let a = SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(None);
        let b = SnowcapNode::<M>::new(SnowcapNodeData::Widget("bar".into())).with_attrs(None);
        assert_eq!(a.compare(&b), SnowcapNodeComparison::DataDiffer);

        let a = SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(None);
        let b = SnowcapNode::<M>::new(SnowcapNodeData::Widget("bar".into())).with_attrs(None);
        assert_eq!(a.compare(&b), SnowcapNodeComparison::DataDiffer);

        let mut attr_a = Attributes::new();
        attr_a
            .push(Attribute::new(crate::attribute::AttributeValue::Label(
                "foo".into(),
            )))
            .ok();

        let mut attr_b = Attributes::new();
        attr_b
            .push(Attribute::new(crate::attribute::AttributeValue::Label(
                "bar".into(),
            )))
            .ok();

        let a =
            SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(Some(attr_a));
        let b =
            SnowcapNode::<M>::new(SnowcapNodeData::Widget("foo".into())).with_attrs(Some(attr_b));
        assert_eq!(a.compare(&b), SnowcapNodeComparison::AttributeDiffer);
    }
}
