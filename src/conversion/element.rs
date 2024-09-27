use iced::{
    widget::{text::IntoFragment, Space, Text},
    Element,
};

use crate::{
    conversion::widget::SnowcapWidget,
    error::Error,
    message::Message,
    parser::{MarkupTree, Value},
};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, stack::SnowcapStack,
};

impl<'a, AppMessage> IntoFragment<'a> for &MarkupTree<AppMessage> {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        match self {
            MarkupTree::Value(value) => match value {
                Value::String(s) => s.clone().into(),
                Value::Number(n) => format!("{n}").into(),
                Value::Boolean(b) => format!("{b}").into(),
                Value::Null => format!("null").into(),
                Value::DataSource {
                    name: _,
                    value: _,
                    provider,
                } => match provider {
                    //DataProvider::File(file_provider) => file_provider.data().clone().into(),
                    _ => "Unsupported DataProvider".into(),
                },
            },
            _ => "Expecting MarkupType::Value".into(),
        }
    }
}

impl<'a, SnowcapMessage, AppMessage> TryInto<Element<'a, SnowcapMessage>>
    for &'a MarkupTree<AppMessage>
where
    SnowcapMessage: 'a + Clone + From<Message<AppMessage>>,
{
    type Error = Error;

    fn try_into(self) -> Result<Element<'a, SnowcapMessage>, Error> {
        match &self {
            MarkupTree::None => Ok(Space::new(0, 0).into()),

            MarkupTree::Element {
                name,
                attrs,
                content,
            } => SnowcapWidget::convert::<SnowcapMessage, AppMessage>(name, attrs, content),

            MarkupTree::Container { content, attrs } => {
                SnowcapContainer::convert::<SnowcapMessage, AppMessage>(attrs, content)
            }

            MarkupTree::Row { attrs, contents } => {
                SnowcapRow::convert::<SnowcapMessage, AppMessage>(attrs, contents)
            }

            MarkupTree::Column { attrs, contents } => {
                SnowcapColumn::convert::<SnowcapMessage, AppMessage>(attrs, contents)
            }

            MarkupTree::Stack { attrs, contents } => {
                SnowcapStack::convert::<SnowcapMessage, AppMessage>(attrs, contents)
            }
            MarkupTree::Label(_) => todo!(),
            MarkupTree::Value(value) => {
                // Convert Values to iced Elements
                match value {
                    Value::String(str) => Ok(Text::new(str.clone()).into()),
                    Value::Number(num) => Ok(Text::new(num).into()),
                    Value::Boolean(val) => Ok(Text::new(val).into()),
                    Value::Null => Ok(Text::new("null").into()),
                    Value::DataSource {
                        name,
                        value,
                        provider: _,
                    } => Ok(Text::new(format!("Data source [{name}:{value}]")).into()),
                }
            }
            _ => todo!(),
        }
    }
}
