use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    tree_util::ChildData,
    NodeId,
};
use iced::{widget::Row, Element};
use tracing::warn;

pub struct SnowcapRow;

impl SnowcapRow {
    pub fn convert<M>(
        attrs: Attributes,
        contents: Option<Vec<ChildData<'static, M>>>,
    ) -> Result<DynamicWidget<'static, M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut row = if let Some(contents) = contents {
            let children: Vec<Element<'_, M>> = contents
                .into_iter()
                .filter_map(|item| {
                    tracing::debug!("Row Child {:?}", item);
                    Some(item.into())
                })
                .collect(); // Convert each item into Element

            Row::with_children(children)
        } else {
            Row::new()
        };

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
                }
            };
        }

        Ok(DynamicWidget::default().with_widget(row))
    }
}
