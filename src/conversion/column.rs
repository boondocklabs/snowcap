use iced::{widget::Column, Element};
use tracing::debug_span;

use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    NodeId, NodeRef,
};

pub struct SnowcapColumn<'a, M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    contents: Vec<DynamicWidget<M>>,
    column: Column<'a, M>,
}

impl<'a, M> SnowcapColumn<'a, M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub fn convert(
        attrs: Attributes,
        contents: &Vec<NodeRef<M>>,
    ) -> Result<Column<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let children: Result<Vec<Element<'static, M>>, ConversionError> = debug_span!("row-item")
            .in_scope(|| {
                (contents)
                    .iter()
                    .map(|item| {
                        tracing::debug!("{:#?}", item);

                        let element = DynamicWidget::from_node(item.clone())?.into_element();
                        Ok(element)
                    })
                    .collect() // Convert each item into Element
            });

        let mut col = Column::with_children(children?);

        for attr in attrs {
            col = match *attr {
                AttributeValue::HorizontalAlignment(horizontal) => col.align_x(horizontal),
                AttributeValue::Padding(padding) => col.padding(padding),
                AttributeValue::WidthLength(length) => col.width(length),
                AttributeValue::HeightLength(length) => col.height(length),
                AttributeValue::Spacing(pixels) => col.spacing(pixels),
                AttributeValue::MaxWidth(length) => col.max_width(length),
                _ => return Err(ConversionError::UnsupportedAttribute(attr, "Column".into())),
            };
        }

        Ok(col)
    }
}
