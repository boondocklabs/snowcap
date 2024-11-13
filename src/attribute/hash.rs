//! Hashing functions for [`iced`] types which do not implement [`std::hash::Hash`]
//! and impl of [`std::hash::Hash`] on [`AttributeValue`]

use std::hash::{Hash, Hasher};

use super::AttributeValue;

fn hash_color<H: Hasher>(color: &iced::Color, state: &mut H) {
    state.write(&color.r.to_le_bytes());
    state.write(&color.g.to_le_bytes());
    state.write(&color.b.to_le_bytes());
    state.write(&color.a.to_le_bytes());
}

fn hash_radius<H: Hasher>(radius: &iced::border::Radius, state: &mut H) {
    state.write(&radius.top_left.to_le_bytes());
    state.write(&radius.top_right.to_le_bytes());
    state.write(&radius.bottom_right.to_le_bytes());
    state.write(&radius.bottom_left.to_le_bytes());
}

fn hash_border<H: Hasher>(border: &iced::Border, state: &mut H) {
    hash_color(&border.color, state);
    hash_radius(&border.radius, state);
    state.write(&border.width.to_le_bytes());
}

fn hash_shadow<H: Hasher>(shadow: &iced::Shadow, state: &mut H) {
    hash_color(&shadow.color, state);
    state.write(&shadow.blur_radius.to_le_bytes());
    state.write(&shadow.offset.x.to_le_bytes());
    state.write(&shadow.offset.y.to_le_bytes());
}

fn hash_padding<H: Hasher>(padding: &iced::Padding, state: &mut H) {
    state.write(&padding.top.to_le_bytes());
    state.write(&padding.right.to_le_bytes());
    state.write(&padding.bottom.to_le_bytes());
    state.write(&padding.left.to_le_bytes());
}

fn hash_pixels<H: Hasher>(pixels: &iced::Pixels, state: &mut H) {
    state.write(&pixels.0.to_le_bytes());
}

fn hash_length<H: Hasher>(length: &iced::Length, state: &mut H) {
    // Hash the discriminant
    std::mem::discriminant(length).hash(state);

    // Hash any inner values
    match length {
        iced::Length::FillPortion(portion) => state.write_u16(*portion),
        iced::Length::Fixed(fixed) => state.write(&fixed.to_le_bytes()),

        // Variants covered by discriminant only hash
        iced::Length::Fill => {}
        iced::Length::Shrink => {}
    }
}

fn hash_gradient<H: Hasher>(gradient: &iced::Gradient, state: &mut H) {
    std::mem::discriminant(gradient).hash(state);
    match gradient {
        iced::Gradient::Linear(linear) => {
            state.write(&linear.angle.0.to_le_bytes());
            for stop in linear.stops {
                if let Some(stop) = stop {
                    hash_color(&stop.color, state);
                    state.write(&stop.offset.to_le_bytes());
                }
            }
        }
    }
}

fn hash_background<H: Hasher>(background: &iced::Background, state: &mut H) {
    std::mem::discriminant(background).hash(state);
    match background {
        iced::Background::Color(color) => hash_color(color, state),
        iced::Background::Gradient(gradient) => hash_gradient(gradient, state),
    }
}

fn hash_direction<H: Hasher>(direction: &iced::widget::scrollable::Direction, state: &mut H) {
    std::mem::discriminant(direction).hash(state);

    // TODO: Hash the scrollbars
}

fn hash_theme<H: Hasher>(theme: &iced::Theme, state: &mut H) {
    std::mem::discriminant(theme).hash(state);

    match theme {
        iced::Theme::Custom(_arc) => {
            tracing::error!("Hashing of custom theme not implemented");
            todo!()
        }
        _ => {}
    }
}

impl std::hash::Hash for AttributeValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            AttributeValue::TextColor(color) => hash_color(color, state),
            AttributeValue::Border(border) => hash_border(border, state),
            AttributeValue::Shadow(shadow) => hash_shadow(shadow, state),
            AttributeValue::HorizontalAlignment(horizontal) => horizontal.hash(state),
            AttributeValue::VerticalAlignment(vertical) => vertical.hash(state),
            AttributeValue::Padding(padding) => hash_padding(padding, state),
            AttributeValue::WidthLength(length) => hash_length(length, state),
            AttributeValue::WidthPixels(pixels) => hash_pixels(pixels, state),
            AttributeValue::MaxWidth(pixels) => hash_pixels(pixels, state),
            AttributeValue::MaxHeight(pixels) => hash_pixels(pixels, state),
            AttributeValue::HeightLength(length) => hash_length(length, state),
            AttributeValue::HeightPixels(pixels) => hash_pixels(pixels, state),
            AttributeValue::Background(background) => hash_background(background, state),
            AttributeValue::Spacing(pixels) => hash_pixels(pixels, state),
            AttributeValue::Size(pixels) => hash_pixels(pixels, state),
            AttributeValue::CellSize(pixels) => hash_pixels(pixels, state),
            AttributeValue::Clip(clip) => clip.hash(state),
            AttributeValue::Toggled(toggled) => toggled.hash(state),
            AttributeValue::Selected(selected) => selected.hash(state),
            AttributeValue::Label(label) => label.hash(state),
            AttributeValue::Theme(theme) => hash_theme(theme, state),
            AttributeValue::Wrapping(wrapping) => wrapping.hash(state),
            AttributeValue::Shaping(shaping) => shaping.hash(state),
            AttributeValue::SliderValue(value) => value.hash(state),
            AttributeValue::ScrollDirection(direction) => hash_direction(direction, state),
            AttributeValue::Module { kind, module } => {
                kind.hash(state);
                module.hash(state);
            }
        }
    }
}
