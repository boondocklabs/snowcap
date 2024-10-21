use iced::{widget::Column, Element};

use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    tree_util::ChildData,
    NodeId,
};

pub struct SnowcapColumn<'a, M>
where
    M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    //contents: Vec<DynamicWidget<M>>,
    column: Column<'a, M>,
}

impl<'a, M> SnowcapColumn<'a, M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub fn convert(
        attrs: Attributes,
        contents: Option<Vec<ChildData<'static, M>>>,
    ) -> Result<DynamicWidget<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut col = if let Some(contents) = contents {
            let children: Vec<Element<'_, M>> = contents
                .into_iter()
                .filter_map(|item| {
                    tracing::debug!("Column Child {:?}", item);
                    Some(item.into())
                })
                .collect(); // Convert each item into Element

            Column::with_children(children)
        } else {
            Column::new()
        };

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

        Ok(DynamicWidget::default().with_widget(col))
    }
}
