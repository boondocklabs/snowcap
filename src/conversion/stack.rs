use iced::widget::Stack;
use tracing::info_span;

use crate::{
    attribute::{AttributeValue, Attributes},
    error::ConversionError,
    message::WidgetMessage,
    NodeRef,
};

pub struct SnowcapStack<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    //contents: Arc<Vec<NodeRef<'a, SnowcapNode<'a, M>, NodeId>>>,
    stack: Stack<'a, M>,
}

impl<'a, M> SnowcapStack<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn convert(
        attrs: Attributes,
        _contents: Vec<&NodeRef<M>>,
    ) -> Result<Stack<'a, M>, ConversionError>
    where
        //SnowcapMessage: Clone + From<Message<AppMessage>> + 'static,
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        let span = info_span!("stack");
        let _span = span.enter();

        /*
        let children: Result<Vec<Element<'a, M>>, ConversionError> = (**contents)
            .iter()
            .map(|item| item.clone().into_element())
            .collect(); // Convert each item into Element

        let mut stack = Stack::with_children(children?);
        */

        let mut stack = Stack::new();

        for attr in attrs {
            stack = match *attr.value() {
                AttributeValue::WidthLength(length) => stack.width(length),
                AttributeValue::HeightLength(length) => stack.height(length),
                _ => return Err(ConversionError::UnsupportedAttribute(attr, "Stack".into())),
            };
        }

        Ok(stack)
    }
}
