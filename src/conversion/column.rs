use std::sync::Arc;

use iced::{widget::Column, Element};

use crate::{
    attribute::Attributes, error::ConversionError, message::WidgetMessage, tree::node::TreeNode,
};

use super::widget::SnowcapWidget;

pub struct SnowcapColumn<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    contents: Arc<Vec<TreeNode<'a, M>>>,
    column: Column<'a, M>,
}

impl<'a, M> SnowcapColumn<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn convert(
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    ) -> Result<Column<'a, M>, ConversionError>
    where
        //SnowcapMessage: Clone + From<Message<AppMessage>> + 'static,
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        let children: Result<Vec<Element<'a, M>>, ConversionError> = (**contents)
            .iter()
            .map(|item| item.clone().into_element())
            .collect(); // Convert each item into Element

        let mut col = Column::with_children(children?);

        for attr in attrs {
            col = match attr.name().as_str() {
                "spacing" => todo!(),
                "padding" => todo!(),
                "width" => {
                    let width: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    col.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    col.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Horizontal, ConversionError> =
                        (&*attr.value()).try_into();
                    col.align_x(align?)
                }
                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            }
        }

        Ok(col)
    }
}
