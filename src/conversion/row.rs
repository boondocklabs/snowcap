use crate::{
    attribute::Attributes, conversion::widget::SnowcapWidget, error::ConversionError,
    message::WidgetMessage, tree::node::TreeNode,
};
use iced::{widget::Row, Element};
use std::sync::Arc;
use tracing::info_span;

pub struct SnowcapRow<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    contents: Arc<Vec<TreeNode<'a, M>>>,
    row: Row<'a, M>,
}

impl<'a, M> SnowcapRow<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn convert(
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    ) -> Result<Row<'a, M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
    {
        let children: Result<Vec<Element<'a, M>>, ConversionError> = info_span!("row-item")
            .in_scope(|| {
                (**contents)
                    .iter()
                    .map(|item| {
                        tracing::info!("{:#?}", item.inner);
                        item.clone().into_element()
                    })
                    .collect() // Convert each item into Element
            });

        let mut row = Row::with_children(children?);

        for attr in attrs {
            row = match attr.name().as_str() {
                "spacing" => {
                    let spacing: Result<iced::Pixels, ConversionError> =
                        (&*attr.value()).try_into();
                    row.spacing(spacing?)
                }
                "padding" => {
                    let padding: Result<iced::Padding, ConversionError> =
                        (&*attr.value()).try_into();
                    row.padding(padding?)
                }
                "width" => {
                    let width: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    row.width(width?)
                }
                "height" => {
                    let height: Result<iced::Length, ConversionError> = (&*attr.value()).try_into();
                    row.height(height?)
                }
                "align" => {
                    let align: Result<iced::alignment::Vertical, ConversionError> =
                        (&*attr.value()).try_into();
                    row.align_y(align?)
                }
                "clip" => todo!(),
                _ => return Err(ConversionError::UnsupportedAttribute(attr.name().clone())),
            };
        }

        //Ok(widget)

        Ok(row)
    }
}
