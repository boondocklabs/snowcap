use iced::widget::Column;

use crate::{
    attribute::{AttributeValue, Attributes},
    cache::WidgetContent,
    dynamic_widget::DynamicWidget,
    error::ConversionError,
};

pub struct SnowcapColumn;

impl SnowcapColumn {
    pub fn convert<M>(
        attrs: Attributes,
        contents: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: std::fmt::Debug + 'static,
    {
        let mut col = Column::with_children(contents);

        for attr in attrs {
            col = match *attr {
                AttributeValue::HorizontalAlignment(horizontal) => col.align_x(horizontal),
                AttributeValue::Padding(padding) => col.padding(padding),
                AttributeValue::WidthLength(length) => col.width(length),
                AttributeValue::HeightLength(length) => col.height(length),
                AttributeValue::WidthPixels(length) => col.width(length),
                AttributeValue::HeightPixels(length) => col.height(length),
                AttributeValue::Spacing(pixels) => col.spacing(pixels),
                AttributeValue::MaxWidth(length) => col.max_width(length),
                AttributeValue::Clip(clip) => col.clip(clip),
                _ => return Err(ConversionError::UnsupportedAttribute(attr, "Column".into())),
            };
        }

        Ok(DynamicWidget::default().with_widget(col))
    }
}
