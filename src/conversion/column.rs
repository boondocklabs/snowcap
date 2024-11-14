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
            col = match attr.value().cloned() {
                Some(AttributeValue::HorizontalAlignment(horizontal)) => col.align_x(horizontal),
                Some(AttributeValue::Padding(padding)) => col.padding(padding),
                Some(AttributeValue::WidthLength(length)) => col.width(length),
                Some(AttributeValue::HeightLength(length)) => col.height(length),
                Some(AttributeValue::WidthPixels(length)) => col.width(length),
                Some(AttributeValue::HeightPixels(length)) => col.height(length),
                Some(AttributeValue::Spacing(pixels)) => col.spacing(pixels),
                Some(AttributeValue::MaxWidth(length)) => col.max_width(length),
                Some(AttributeValue::Clip(clip)) => col.clip(clip),
                _ => return Err(ConversionError::UnsupportedAttribute(attr, "Column".into())),
            };
        }

        Ok(DynamicWidget::default().with_widget(col))
    }
}
