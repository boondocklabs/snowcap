use iced::{
    widget::{Space, Text},
    Element,
};

use crate::{
    conversion::widget::SnowcapWidget,
    message::Message,
    parser::{MarkupTree, TreeNode, Value},
    ConversionError,
};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, stack::SnowcapStack,
};

impl<'a, SnowcapMessage, AppMessage> TryInto<Element<'a, SnowcapMessage>>
    for &'a TreeNode<AppMessage>
where
    SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
    AppMessage: 'a + Clone + std::fmt::Debug,
{
    type Error = ConversionError;

    fn try_into(self) -> Result<Element<'a, SnowcapMessage>, Self::Error> {
        match &*self.inner() {
            MarkupTree::None => Ok(Space::new(0, 0).into()),

            MarkupTree::Widget {
                element_id: _,
                name,
                attrs,
                content,
            } => SnowcapWidget::convert::<SnowcapMessage, AppMessage>(self, name, attrs, content),

            MarkupTree::Container { content, attrs } => {
                SnowcapContainer::convert::<SnowcapMessage, AppMessage>(attrs, content)
            }

            MarkupTree::Row {
                element_id: _,
                attrs,
                contents,
            } => SnowcapRow::convert::<SnowcapMessage, AppMessage>(attrs, contents),

            MarkupTree::Column {
                element_id: _,
                attrs,
                contents,
            } => SnowcapColumn::convert::<SnowcapMessage, AppMessage>(attrs, contents),

            MarkupTree::Stack {
                element_id: _,
                attrs,
                contents,
            } => SnowcapStack::convert::<SnowcapMessage, AppMessage>(attrs, contents),
            MarkupTree::Value(value) => {
                // Convert Values to iced Elements
                let val = match &*value.borrow() {
                    Value::String(str) => Text::new(str.clone()).into(),
                    Value::Number(num) => Text::new(num).into(),
                    Value::Boolean(val) => Text::new(val).into(),

                    // TODO: We could return an element for known data types
                    Value::Data { .. } => Text::new(format!("Data")).into(),
                    Value::Array(_value) => todo!(),
                };

                Ok(val)
            }
            _ => unimplemented!("Unhandled markup node conversion"),
        }
    }
}
