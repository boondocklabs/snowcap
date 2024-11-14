use crate::{attribute::AttributeValue, cache::WidgetContent};
use iced::widget::Container;

use crate::{attribute::Attributes, dynamic_widget::DynamicWidget, error::ConversionError};

pub struct SnowcapContainer;

impl SnowcapContainer {
    pub fn new<M>(
        attrs: Attributes,
        content: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError>
    where
        M: std::fmt::Debug + 'static,
    {
        let mut container = Container::new(content);
        let mut style = iced::widget::container::Style::default();

        for attr in attrs {
            (container, style) = match attr.value().cloned() {
                Some(AttributeValue::TextColor(color)) => (container, style.color(color)),
                Some(AttributeValue::Border(border)) => (container, style.border(border)),
                Some(AttributeValue::Shadow(shadow)) => (container, style.shadow(shadow)),
                Some(AttributeValue::Background(background)) => {
                    (container, style.background(background))
                }
                Some(AttributeValue::HorizontalAlignment(horizontal)) => {
                    (container.align_x(horizontal), style)
                }
                Some(AttributeValue::VerticalAlignment(vertical)) => {
                    (container.align_y(vertical), style)
                }
                Some(AttributeValue::Padding(padding)) => (container.padding(padding), style),
                Some(AttributeValue::MaxWidth(pixels)) => (container.max_width(pixels), style),
                Some(AttributeValue::WidthLength(length)) => (container.width(length), style),
                Some(AttributeValue::HeightLength(length)) => (container.height(length), style),
                Some(AttributeValue::WidthPixels(pixels)) => (container.width(pixels), style),
                Some(AttributeValue::HeightPixels(pixels)) => (container.height(pixels), style),
                Some(AttributeValue::Clip(clip)) => (container.clip(clip), style),
                _ => {
                    return Err(ConversionError::UnsupportedAttribute(
                        attr,
                        "Container".into(),
                    ))
                }
            };
        }

        container = container.style(move |_theme| style);

        Ok(DynamicWidget::default().with_widget(container))
    }
}
