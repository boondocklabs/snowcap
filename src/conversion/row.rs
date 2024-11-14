use crate::{
    attribute::{AttributeValue, Attributes},
    cache::WidgetContent,
    dynamic_widget::DynamicWidget,
    error::ConversionError,
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
        M: std::fmt::Debug + 'static,
    {
        let mut row = Row::with_children(contents);

        for attr in attrs {
            row = match attr.value().cloned() {
                Some(AttributeValue::VerticalAlignment(vertical)) => row.align_y(vertical),

                // TODO: Clean this up. align:center creates a HorizontalAlignment::Center attribute
                Some(AttributeValue::HorizontalAlignment(_)) => {
                    row.align_y(iced::alignment::Vertical::Center)
                }
                Some(AttributeValue::Padding(padding)) => row.padding(padding),
                Some(AttributeValue::WidthLength(length)) => row.width(length),
                Some(AttributeValue::HeightLength(length)) => row.height(length),
                Some(AttributeValue::WidthPixels(pixels)) => row.width(pixels),
                Some(AttributeValue::HeightPixels(pixels)) => row.height(pixels),
                Some(AttributeValue::Spacing(pixels)) => row.spacing(pixels),
                Some(AttributeValue::Clip(clip)) => row.clip(clip),
                _ => {
                    warn!("Unsupported Row attribute {:#?}", attr);
                    row
                }
            };
        }

        Ok(DynamicWidget::default().with_widget(row))
    }
}
