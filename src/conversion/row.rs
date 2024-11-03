use crate::{
    attribute::{AttributeValue, Attributes},
    cache::WidgetContent,
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    NodeId,
};
use iced::widget::Row;
use tracing::warn;

pub struct SnowcapRow;

impl SnowcapRow {
    pub fn convert<M>(
        attrs: Attributes,
        contents: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut row = Row::with_children(contents);

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
