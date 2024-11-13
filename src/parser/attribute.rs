use iced::widget::scrollable::Scrollbar;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use tracing::{debug, debug_span, warn};

use crate::{
    attribute::{Attribute, AttributeKind, AttributeValue, Attributes},
    module::argument::ModuleArgument,
    parser::{color::ColorParser, gradient::GradientParser, module::ModuleParser, ParserContext},
};

use super::{ParseError, Value};

#[derive(Debug)]
enum AttributeOption {
    Color(iced::Color),
    Gradient(iced::Gradient),
    WidthPixels(iced::Pixels),
    Radius(iced::border::Radius),
}

#[derive(Parser)]
#[grammar = "parser/attribute.pest"]
pub struct AttributeParser;

impl AttributeParser {
    fn parse_background(pairs: Pairs<'_, Rule>) -> Result<AttributeValue, ParseError> {
        let options = Self::parse_options(pairs)?;

        for option in options {
            match option {
                AttributeOption::Color(color) => {
                    return Ok(AttributeValue::Background(iced::Background::Color(color)))
                }
                AttributeOption::Gradient(gradient) => {
                    return Ok(AttributeValue::Background(iced::Background::Gradient(
                        gradient,
                    )))
                }
                _ => warn!("Unsupported background option {:?}", option),
            }
        }

        Err(ParseError::InvalidColor("parse_background".into()))
    }

    fn parse_alignment(pair: Pair<'_, Rule>) -> Result<AttributeValue, ParseError> {
        match pair.as_rule() {
            Rule::horizontal => match pair.into_inner().last().unwrap().as_rule() {
                Rule::left => Ok(AttributeValue::HorizontalAlignment(
                    iced::alignment::Horizontal::Left,
                )),
                Rule::center => Ok(AttributeValue::HorizontalAlignment(
                    iced::alignment::Horizontal::Center,
                )),
                Rule::right => Ok(AttributeValue::HorizontalAlignment(
                    iced::alignment::Horizontal::Right,
                )),
                _ => panic!("unknown horizontal alignment value"),
            },
            Rule::vertical => match pair.into_inner().last().unwrap().as_rule() {
                Rule::top => Ok(AttributeValue::VerticalAlignment(
                    iced::alignment::Vertical::Top,
                )),
                Rule::center => Ok(AttributeValue::VerticalAlignment(
                    iced::alignment::Vertical::Center,
                )),
                Rule::bottom => Ok(AttributeValue::VerticalAlignment(
                    iced::alignment::Vertical::Bottom,
                )),
                _ => panic!("unknown horizontal alignment value"),
            },
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "attr_align_x {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    fn parse_string(pair: Pair<'_, Rule>) -> Result<String, ParseError> {
        match pair.as_rule() {
            Rule::string => {
                let str = pair.into_inner().last().unwrap().as_str().to_string();
                debug!("parse_string() inner '{str}'");
                Ok(str)
            }
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "parse_string expecting string, got {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    fn parse_boolean(pair: Pair<'_, Rule>) -> Result<bool, ParseError> {
        match pair.as_rule() {
            Rule::boolean => Ok(pair.as_str().parse().map_err(|e| ParseError::Boolean(e))?),
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "parse_float expecting float, got {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    fn parse_float(pair: Pair<'_, Rule>) -> Result<f32, ParseError> {
        match pair.as_rule() {
            Rule::float => Ok(pair.as_str().parse().map_err(|e| ParseError::Float(e))?),
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "parse_float expecting float, got {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    fn parse_u16(pair: Pair<'_, Rule>) -> Result<u16, ParseError> {
        match pair.as_rule() {
            Rule::integer => Ok(pair.as_str().parse().map_err(|e| ParseError::Integer(e))?),
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "parse_u16 expecting integer, got {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    fn parse_float_list(pairs: Pairs<'_, Rule>) -> Result<Vec<f32>, ParseError> {
        let mut list = Vec::new();

        for pair in pairs {
            list.push(Self::parse_float(pair)?)
        }

        Ok(list)
    }

    fn parse_padding(pair: Pair<'_, Rule>) -> Result<iced::Padding, ParseError> {
        let mut padding = iced::Padding::default();

        for pair in pair.into_inner() {
            padding = match pair.as_rule() {
                Rule::uniform => {
                    let padding: f32 = pair.as_str().parse().unwrap();
                    iced::Padding::new(padding)
                }
                Rule::edge => {
                    let vals = Self::parse_float_list(pair.into_inner())?;
                    padding
                        .top(vals[0])
                        .bottom(vals[0])
                        .left(vals[1])
                        .right(vals[1])
                }
                Rule::full => {
                    let vals = Self::parse_float_list(pair.into_inner())?;
                    padding
                        .top(vals[0])
                        .right(vals[1])
                        .bottom(vals[2])
                        .left(vals[3])
                }
                Rule::option_top => {
                    padding.top(Self::parse_float(pair.into_inner().last().unwrap())?)
                }
                Rule::option_right => {
                    padding.right(Self::parse_float(pair.into_inner().last().unwrap())?)
                }
                Rule::option_bottom => {
                    padding.bottom(Self::parse_float(pair.into_inner().last().unwrap())?)
                }
                Rule::option_left => {
                    padding.left(Self::parse_float(pair.into_inner().last().unwrap())?)
                }
                Rule::delimiter => continue,
                _ => {
                    return Err(ParseError::UnsupportedRule(format!(
                        "Padding unsupported rule: {:?}",
                        pair.as_rule()
                    )))
                }
            };
        }
        Ok(padding)
    }

    fn parse_wrapping(pair: Pair<'_, Rule>) -> Result<iced::widget::text::Wrapping, ParseError> {
        match pair.as_rule() {
            Rule::glyph => Ok(iced::widget::text::Wrapping::Glyph),
            Rule::word => Ok(iced::widget::text::Wrapping::Word),
            Rule::either => Ok(iced::widget::text::Wrapping::WordOrGlyph),
            Rule::none => Ok(iced::widget::text::Wrapping::None),
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_wrapping() expecting glpyh | word | either | none. Got {:#?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_shaping(pair: Pair<'_, Rule>) -> Result<iced::widget::text::Shaping, ParseError> {
        match pair.as_rule() {
            Rule::basic => Ok(iced::widget::text::Shaping::Basic),
            Rule::advanced => Ok(iced::widget::text::Shaping::Advanced),
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_shaping() expecting basic | advanced. Got {:#?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_direction(
        pair: Pair<'_, Rule>,
    ) -> Result<iced::widget::scrollable::Direction, ParseError> {
        match pair.as_rule() {
            Rule::direction_horizontal => Ok(iced::widget::scrollable::Direction::Horizontal(
                Scrollbar::default(),
            )),
            Rule::direction_vertical => Ok(iced::widget::scrollable::Direction::Vertical(
                Scrollbar::default(),
            )),
            Rule::both => Ok(iced::widget::scrollable::Direction::Both {
                vertical: Scrollbar::default(),
                horizontal: Scrollbar::default(),
            }),
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_shaping() expecting basic | advanced. Got {:#?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_radius(pair: Pair<'_, Rule>) -> Result<iced::border::Radius, ParseError> {
        match pair.as_rule() {
            Rule::uniform => {
                debug!("Radius Uniform {}", pair.as_str());
                let r: f32 = pair.as_str().parse().unwrap();
                Ok(iced::border::radius(r))
            }
            Rule::full => {
                debug!("Radius Full {}", pair.as_str());
                let vals = Self::parse_float_list(pair.into_inner())?;
                let radius = iced::border::Radius::default()
                    .top_left(vals[0])
                    .top_right(vals[1])
                    .bottom_right(vals[2])
                    .bottom_left(vals[3]);
                Ok(radius)
            }
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_radius() expecting uniform | full, got {:?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_pixels(pair: Pair<'_, Rule>) -> Result<iced::Pixels, ParseError> {
        match pair.as_rule() {
            Rule::float => Ok(iced::Pixels(
                pair.as_str().parse().map_err(|e| ParseError::Float(e))?,
            )),
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_pixels expecting float got {:?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_length(pair: Pair<'_, Rule>) -> Result<iced::Length, ParseError> {
        match pair.as_rule() {
            Rule::fill => Ok(iced::Length::Fill),
            Rule::shrink => Ok(iced::Length::Shrink),
            Rule::fixed => {
                let fixed = Self::parse_float(pair.into_inner().last().unwrap())?;
                Ok(iced::Length::Fixed(fixed))
            }
            Rule::fill_portion => {
                let portion = Self::parse_u16(pair.into_inner().last().unwrap())?;
                Ok(iced::Length::FillPortion(portion))
            }
            _ => Err(ParseError::UnsupportedRule(format!(
                "parse_length expecting fill | fixed | shrink | fill_portion, got {:?}",
                pair.as_rule()
            ))),
        }
    }

    /// Get the [`AttributeKind`] for a pair
    fn pair_kind(pair: &Pair<'_, Rule>) -> Result<AttributeKind, ParseError> {
        match pair.as_rule() {
            Rule::attr_background => Ok(AttributeKind::Background),
            Rule::attr_text_color => Ok(AttributeKind::TextColor),
            Rule::attr_align_x => Ok(AttributeKind::HorizontalAlignment),
            Rule::attr_align_y => Ok(AttributeKind::VerticalAlignment),
            Rule::attr_padding => Ok(AttributeKind::Padding),
            Rule::attr_height => Ok(AttributeKind::HeightPixels),
            Rule::attr_width => Ok(AttributeKind::WidthPixels),
            Rule::attr_spacing => Ok(AttributeKind::Spacing),
            Rule::attr_size => Ok(AttributeKind::Size),
            Rule::attr_cell_size => Ok(AttributeKind::CellSize),
            Rule::attr_selected => Ok(AttributeKind::Selected),
            Rule::attr_label => Ok(AttributeKind::Label),
            Rule::attr_max_width => Ok(AttributeKind::MaxWidth),
            Rule::attr_max_height => Ok(AttributeKind::MaxHeight),
            Rule::attr_align => Ok(AttributeKind::HorizontalAlignment),
            Rule::attr_clip => Ok(AttributeKind::Clip),
            Rule::attr_toggled => Ok(AttributeKind::Toggled),
            Rule::attr_wrapping => Ok(AttributeKind::Wrapping),
            Rule::attr_shaping => Ok(AttributeKind::Shaping),
            Rule::attr_border => Ok(AttributeKind::Border),
            Rule::attr_shadow => Ok(AttributeKind::Shadow),
            Rule::attr_direction => Ok(AttributeKind::ScrollDirection),
            _ => Err(ParseError::UnsupportedRule(format!(
                "In pair_kind() rule={:?} {}:{}",
                pair.as_rule(),
                file!(),
                line!()
            ))),
        }
    }

    fn parse_attribute(pair: Pair<'_, Rule>) -> Result<Option<AttributeValue>, ParseError> {
        // Check if this pair contains a module
        let mut inner = pair.clone().into_inner();
        let module = inner.find(|pair| {
            if pair.as_rule() == Rule::module {
                true
            } else {
                false
            }
        });

        // If we found a module, parse it and return [`AttributeValue::Module`]
        if let Some(module) = module {
            let kind = Self::pair_kind(&pair)?;
            let mut module = ModuleParser::parse_str(module.as_str(), ParserContext::default())?;

            // Insert module arguments
            module.args_mut().insert(ModuleArgument::new(
                "_attribute".into(),
                Value::new_attribute_kind(kind),
            ));

            return Ok(Some(AttributeValue::Module { kind, module }));
        }

        match pair.as_rule() {
            Rule::attr_background => Ok(Some(Self::parse_background(pair.into_inner())?)),
            Rule::attr_text_color => {
                let color = ColorParser::parse_str(pair.into_inner().as_str())?;
                Ok(Some(AttributeValue::TextColor(color)))
            }
            Rule::attr_align_x | Rule::attr_align_y => {
                if let Some(pair) = pair.into_inner().last() {
                    Ok(Some(Self::parse_alignment(pair)?))
                } else {
                    todo!();
                }
            }
            Rule::attr_padding => Ok(Some(AttributeValue::Padding(Self::parse_padding(pair)?))),
            Rule::attr_height => {
                let pair = pair.into_inner().last().unwrap();

                match pair.as_rule() {
                    Rule::pixels => Ok(Some(AttributeValue::HeightPixels(Self::parse_pixels(
                        pair.into_inner().last().unwrap(),
                    )?))),

                    Rule::length => Ok(Some(AttributeValue::HeightLength(Self::parse_length(
                        pair.into_inner().last().unwrap(),
                    )?))),

                    _ => Err(ParseError::UnsupportedRule(format!(
                        "attr_height expecting pixels | length, got {:?}",
                        pair.as_rule()
                    ))),
                }
            }
            Rule::attr_width => {
                let pair = pair.into_inner().last().unwrap();

                match pair.as_rule() {
                    Rule::pixels => Ok(Some(AttributeValue::WidthPixels(Self::parse_pixels(
                        pair.into_inner().last().unwrap(),
                    )?))),

                    Rule::length => Ok(Some(AttributeValue::WidthLength(Self::parse_length(
                        pair.into_inner().last().unwrap(),
                    )?))),

                    _ => Err(ParseError::UnsupportedRule(format!(
                        "attr_height expecting pixels | length, got {:?}",
                        pair.as_rule()
                    ))),
                }
            }
            Rule::attr_align => {
                if let Some(pair) = pair.into_inner().last() {
                    Ok(Some(Self::parse_alignment(pair)?))
                } else {
                    todo!();
                }
            }
            Rule::attr_spacing => Ok(Some(AttributeValue::Spacing(Self::parse_pixels(
                pair.into_inner()
                    .last()
                    .unwrap()
                    .into_inner()
                    .last()
                    .unwrap(),
            )?))),
            Rule::attr_size => Ok(Some(AttributeValue::Size(Self::parse_pixels(
                pair.into_inner()
                    .last()
                    .unwrap()
                    .into_inner()
                    .last()
                    .unwrap(),
            )?))),
            Rule::attr_cell_size => Ok(Some(AttributeValue::CellSize(Self::parse_pixels(
                pair.into_inner()
                    .last()
                    .unwrap()
                    .into_inner()
                    .last()
                    .unwrap(),
            )?))),
            Rule::attr_selected => Ok(Some(AttributeValue::Selected(Self::parse_string(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_label => Ok(Some(AttributeValue::Label(Self::parse_string(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_toggled => Ok(Some(AttributeValue::Toggled(Self::parse_boolean(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_clip => Ok(Some(AttributeValue::Clip(Self::parse_boolean(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_border => {
                let mut border = iced::Border::default();
                let options = Self::parse_options(pair.into_inner())?;
                for option in options {
                    border = match option {
                        AttributeOption::Color(color) => border.color(color),
                        AttributeOption::WidthPixels(pixels) => border.width(pixels),
                        AttributeOption::Radius(radius) => border.rounded(radius),
                        _ => {
                            warn!("Unsupported Border option {:?}", option);
                            border
                        }
                    };
                }

                Ok(Some(AttributeValue::Border(border)))
            }
            Rule::attr_wrapping => Ok(Some(AttributeValue::Wrapping(Self::parse_wrapping(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_shaping => Ok(Some(AttributeValue::Shaping(Self::parse_shaping(
                pair.into_inner().last().unwrap(),
            )?))),
            Rule::attr_direction => Ok(Some(AttributeValue::ScrollDirection(
                Self::parse_direction(pair.into_inner().last().unwrap())?,
            ))),
            Rule::EOI => Ok(None),
            _ => Err(ParseError::UnsupportedRule(format!(
                "In parse_attribute rule={:?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_options(pairs: Pairs<'_, Rule>) -> Result<Vec<AttributeOption>, ParseError> {
        let mut options = Vec::new();

        for pair in pairs {
            debug!("OPTION {:?}", pair.as_rule());
            match pair.as_rule() {
                Rule::option_color => {
                    let color = ColorParser::parse_str(pair.into_inner().as_str())?;
                    options.push(AttributeOption::Color(color));
                }
                Rule::option_gradient => {
                    let gradient = GradientParser::parse_str(pair.into_inner().as_str())?;
                    options.push(AttributeOption::Gradient(gradient));
                }
                Rule::option_width => {
                    let width = Self::parse_pixels(pair.into_inner().last().unwrap())?;
                    options.push(AttributeOption::WidthPixels(width.into()));
                }
                Rule::option_radius => {
                    let radius = Self::parse_radius(pair.into_inner().last().unwrap())?;
                    options.push(AttributeOption::Radius(radius))
                }
                _ => {}
            };
        }

        Ok(options)
    }

    pub fn parse_attributes(data: &str) -> Result<Attributes, ParseError> {
        let attributes: Result<Attributes, ParseError> =
            debug_span!("AttributeParser").in_scope(|| {
                debug!("Parsing attributes '{data}'");
                let mut attributes = Attributes::default();

                let pairs = AttributeParser::parse(Rule::attribute_list, data)?;

                for pair in pairs {
                    debug!("{:?}", pair.as_rule());
                    match pair.as_rule() {
                        Rule::attribute_list => {
                            for pair in pair.into_inner() {
                                //debug!("{:#?}", pair);
                                if let Some(value) = Self::parse_attribute(pair)? {
                                    attributes.push(Attribute::new(value)).unwrap();
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => continue,
                    }
                }

                debug!("Parsed Attributes {:#?}", attributes);

                Ok(attributes)
            });

        attributes
    }
}

#[cfg(test)]
mod tests {
    use iced::{
        widget::{
            scrollable::{Direction, Scrollbar},
            text::{Shaping, Wrapping},
        },
        Padding,
    };
    use tracing::info;
    use tracing_test::traced_test;

    use crate::attribute::AttributeKind;

    use super::*;

    #[traced_test]
    #[test]
    fn test_basic() {
        let attrs = AttributeParser::parse_attributes("toggled:true").unwrap();
        assert!(attrs.get(AttributeKind::Toggled).unwrap().is_some());
        let attrs = AttributeParser::parse_attributes("toggled:false").unwrap();
        assert!(attrs.get(AttributeKind::Toggled).unwrap().is_some());

        let attrs = AttributeParser::parse_attributes("align-x:left").unwrap();
        assert!(attrs
            .get(AttributeKind::HorizontalAlignment)
            .unwrap()
            .is_some());
        let attrs = AttributeParser::parse_attributes("align-x:right").unwrap();
        assert!(attrs
            .get(AttributeKind::HorizontalAlignment)
            .unwrap()
            .is_some());
        let attrs = AttributeParser::parse_attributes("align-x:center").unwrap();
        assert!(attrs
            .get(AttributeKind::HorizontalAlignment)
            .unwrap()
            .is_some());

        let attrs = AttributeParser::parse_attributes("align-y:top").unwrap();
        assert!(attrs
            .get(AttributeKind::VerticalAlignment)
            .unwrap()
            .is_some());
        let attrs = AttributeParser::parse_attributes("align-y:center").unwrap();
        assert!(attrs
            .get(AttributeKind::VerticalAlignment)
            .unwrap()
            .is_some());
        let attrs = AttributeParser::parse_attributes("align-y:bottom").unwrap();
        assert!(attrs
            .get(AttributeKind::VerticalAlignment)
            .unwrap()
            .is_some());
    }

    fn check_radius(
        attr: &AttributeValue,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) {
        match attr {
            AttributeValue::Border(border) => {
                assert!(border.radius.top_left == top_left);
                assert!(border.radius.top_right == top_right);
                assert!(border.radius.bottom_right == bottom_right);
                assert!(border.radius.bottom_left == bottom_left);
                info!("{:#?}", border);
            }
            _ => panic!("Border AttributeValue not found"),
        }
    }

    #[traced_test]
    #[test]
    fn test_padding() {
        let attrs = AttributeParser::parse_attributes("padding:1").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 1.0,
                right: 1.0,
                bottom: 1.0,
                left: 1.0
            })
        );

        let attrs = AttributeParser::parse_attributes("padding:top(1)").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 1.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0
            })
        );

        let attrs = AttributeParser::parse_attributes("padding:right(1)").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 0.0,
                right: 1.0,
                bottom: 0.0,
                left: 0.0
            })
        );

        let attrs = AttributeParser::parse_attributes("padding:bottom(1)").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 1.0,
                left: 0.0
            })
        );

        let attrs = AttributeParser::parse_attributes("padding:left(1)").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 1.0
            })
        );

        let attrs = AttributeParser::parse_attributes("padding:left(1), right(2)").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Padding).unwrap().unwrap(),
            AttributeValue::Padding(Padding {
                top: 0.0,
                right: 2.0,
                bottom: 0.0,
                left: 1.0
            })
        );
    }

    #[traced_test]
    #[test]
    fn test_radius() {
        let attrs = AttributeParser::parse_attributes("border:radius(1.0)").unwrap();
        let attr = attrs.get(AttributeKind::Border).unwrap().unwrap();
        check_radius(&attr, 1.0, 1.0, 1.0, 1.0);

        let attrs = AttributeParser::parse_attributes("border:radius(1.0, 2.0, 3.0, 4.0)").unwrap();
        let attr = attrs.get(AttributeKind::Border).unwrap().unwrap();
        check_radius(&attr, 1.0, 2.0, 3.0, 4.0);
    }

    #[traced_test]
    #[test]
    fn test_shaping() {
        let attrs = AttributeParser::parse_attributes("shaping:basic").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Shaping).unwrap().unwrap(),
            AttributeValue::Shaping(Shaping::Basic)
        );

        let attrs = AttributeParser::parse_attributes("shaping:advanced").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Shaping).unwrap().unwrap(),
            AttributeValue::Shaping(Shaping::Advanced)
        );
    }

    #[traced_test]
    #[test]
    fn test_wrapping() {
        let attrs = AttributeParser::parse_attributes("wrapping:none").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Wrapping).unwrap().unwrap(),
            AttributeValue::Wrapping(Wrapping::None)
        );

        let attrs = AttributeParser::parse_attributes("wrapping:glyph").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Wrapping).unwrap().unwrap(),
            AttributeValue::Wrapping(Wrapping::Glyph)
        );

        let attrs = AttributeParser::parse_attributes("wrapping:word").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Wrapping).unwrap().unwrap(),
            AttributeValue::Wrapping(Wrapping::Word)
        );

        let attrs = AttributeParser::parse_attributes("wrapping:either").unwrap();
        assert_eq!(
            attrs.get(AttributeKind::Wrapping).unwrap().unwrap(),
            AttributeValue::Wrapping(Wrapping::WordOrGlyph)
        );
    }

    #[traced_test]
    #[test]
    fn test_border() {
        let attrs =
            AttributeParser::parse_attributes("border:color(1.0,1.0,1.0),width(2.0),radius(1.0)")
                .unwrap();

        let attr = attrs.get(AttributeKind::Border).unwrap().unwrap();

        check_radius(&attr, 1.0, 1.0, 1.0, 1.0);

        match attr {
            AttributeValue::Border(border) => {
                assert!(border.width == 2.0);
                info!("{:#?}", border);
            }
            _ => panic!("Border AttributeValue not found"),
        }
    }

    #[traced_test]
    #[test]
    fn test_direction() {
        let attrs = AttributeParser::parse_attributes("direction: horizontal").unwrap();
        let attr = attrs.get(AttributeKind::ScrollDirection).unwrap().unwrap();

        match attr {
            AttributeValue::ScrollDirection(direction) => {
                assert!(direction == Direction::Horizontal(Scrollbar::default()))
            }
            _ => panic!("ScrollDirection AttributeValue not found"),
        }

        let attrs = AttributeParser::parse_attributes("direction: vertical").unwrap();
        let attr = attrs.get(AttributeKind::ScrollDirection).unwrap().unwrap();

        match attr {
            AttributeValue::ScrollDirection(direction) => {
                assert!(direction == Direction::Vertical(Scrollbar::default()))
            }
            _ => panic!("ScrollDirection AttributeValue not found"),
        }

        let attrs = AttributeParser::parse_attributes("direction: both").unwrap();
        let attr = attrs.get(AttributeKind::ScrollDirection).unwrap().unwrap();

        match attr {
            AttributeValue::ScrollDirection(direction) => {
                assert!(
                    direction
                        == Direction::Both {
                            vertical: Scrollbar::default(),
                            horizontal: Scrollbar::default()
                        }
                )
            }
            _ => panic!("ScrollDirection AttributeValue not found"),
        }
    }

    #[traced_test]
    #[test]
    fn test_clip() {
        let attrs = AttributeParser::parse_attributes("clip: true").unwrap();
        let attr = attrs.get(AttributeKind::Clip).unwrap().unwrap();

        match attr {
            AttributeValue::Clip(clip) => {
                assert_eq!(clip, true)
            }
            _ => panic!("Clip AttributeValue not found"),
        }

        let attrs = AttributeParser::parse_attributes("clip: false").unwrap();
        let attr = attrs.get(AttributeKind::Clip).unwrap().unwrap();

        match attr {
            AttributeValue::Clip(clip) => {
                assert_eq!(clip, false)
            }
            _ => panic!("Clip AttributeValue not found"),
        }
    }
}
