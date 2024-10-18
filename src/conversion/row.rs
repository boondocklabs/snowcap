use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    NodeId, NodeRef,
};
use iced::{widget::Row, Element};
use tracing::{debug, debug_span, warn};

pub struct SnowcapRow<'a, M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'a,
{
    //contents: Arc<Vec<TreeNode<'a, SnowcapNode<'a, M>, NodeId>>>,
    row: Row<'a, M>,
}

impl<'a, M> SnowcapRow<'a, M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub fn convert(
        attrs: Attributes,
        contents: &Vec<NodeRef<M>>,
    ) -> Result<Row<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let children: Result<Vec<Element<'static, M>>, ConversionError> = debug_span!("row-item")
            .in_scope(|| {
                (contents)
                    .iter()
                    .map(|item| {
                        debug!("{:#?}", item);

                        let element = DynamicWidget::from_node(item.clone())?.into_element();
                        Ok(element)
                    })
                    .collect() // Convert each item into Element
            });

        let mut row = Row::with_children(children?);

        for attr in attrs {
            row = match *attr {
                AttributeValue::VerticalAlignment(vertical) => row.align_y(vertical),

                // TODO: Clean this up. align:center creates a HorizontalAlignment::Center attribute
                AttributeValue::HorizontalAlignment(_) => {
                    row.align_y(iced::alignment::Vertical::Center)
                }
                AttributeValue::Padding(padding) => row.padding(padding),
                AttributeValue::WidthLength(length) => row.width(length),
                AttributeValue::HeightLength(length) => row.height(length),
                AttributeValue::WidthPixels(pixels) => row.width(pixels),
                AttributeValue::HeightPixels(pixels) => row.height(pixels),
                AttributeValue::Spacing(pixels) => row.spacing(pixels),
                AttributeValue::Clip(clip) => row.clip(clip),
                _ => {
                    warn!("Unsupported Row attribute {:#?}", attr);
                    row
                } //_ => return Err(ConversionError::UnsupportedAttribute(attr, "Row".into())),
            };
        }

        Ok(row)
    }
}
