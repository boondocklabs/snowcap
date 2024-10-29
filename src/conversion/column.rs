use iced::{widget::Column, Element};

use crate::{
    attribute::{AttributeValue, Attributes},
    dynamic_widget::DynamicWidget,
    error::ConversionError,
    message::WidgetMessage,
    tree_util::WidgetContent,
    NodeId,
};

pub struct SnowcapColumn;

impl SnowcapColumn {
    pub fn convert<M>(
        attrs: Attributes,
        contents: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
    {
        let mut col = Column::with_children(contents);

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
