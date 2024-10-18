use iced::Background;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use tracing::{debug, debug_span};

use crate::{
    attribute::{Attribute, AttributeValue, Attributes},
    parser::{color::ColorParser, gradient::GradientParser},
};

use super::ParseError;

#[derive(Parser)]
#[grammar = "parser/attribute.pest"]
pub struct AttributeParser;

impl AttributeParser {
    fn parse_background(pairs: Pairs<'_, Rule>) -> Result<AttributeValue, ParseError> {
        for pair in pairs {
            match pair.as_rule() {
                Rule::gradient => {
                    let gradient = GradientParser::parse_str(pair.as_str())?;
                    return Ok(AttributeValue::Background(Background::Gradient(gradient)));
                }
                Rule::color => {
                    let color = ColorParser::parse_str(pair.as_str())?;
                    return Ok(AttributeValue::Background(Background::Color(color)));
                }
                _ => continue,
            }
        }

        Err(ParseError::InvalidColor("parse_background".into()))
    }

    fn parse_alignment(pair: Pair<'_, Rule>) -> Result<AttributeValue, ParseError> {
        match pair.as_rule() {
            Rule::horizontal => {
                debug!("HORIZONTAL");
                match pair.into_inner().last().unwrap().as_rule() {
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
                }
            }
            Rule::vertical => {
                debug!("VERTICAL");
                match pair.into_inner().last().unwrap().as_rule() {
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
                }
            }
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
            Rule::string => Ok(pair.as_str().into()),
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

    fn parse_padding(pair: Pair<'_, Rule>) -> Result<AttributeValue, ParseError> {
        match pair.as_rule() {
            Rule::padding_uniform => {
                let padding: f32 = pair.as_str().parse().unwrap();
                Ok(AttributeValue::Padding(iced::Padding::new(padding)))
            }
            Rule::padding_edge => {
                let vals = Self::parse_float_list(pair.into_inner())?;
                let padding = iced::Padding::new(0.0)
                    .top(vals[0])
                    .bottom(vals[0])
                    .left(vals[1])
                    .right(vals[1]);
                Ok(AttributeValue::Padding(padding))
            }
            Rule::padding_full => {
                let vals = Self::parse_float_list(pair.into_inner())?;
                let padding = iced::Padding::new(0.0)
                    .top(vals[0])
                    .right(vals[1])
                    .bottom(vals[2])
                    .left(vals[3]);
                Ok(AttributeValue::Padding(padding))
            }
            _ => Err(ParseError::UnsupportedRule(format!(
                "attr_align_x {:?}",
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

    fn parse_attribute(pair: Pair<'_, Rule>) -> Result<Option<AttributeValue>, ParseError> {
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
            Rule::attr_padding => {
                if let Some(pair) = pair.into_inner().last() {
                    Ok(Some(Self::parse_padding(pair)?))
                } else {
                    todo!();
                }
            }
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
            Rule::EOI => Ok(None),
            _ => Err(ParseError::UnsupportedRule(format!(
                "In parse_attribute rule={:?}",
                pair.as_rule()
            ))),
        }
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
                                debug!("{:#?}", pair);
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

                debug!("Attributes {:#?}", attributes);

                Ok(attributes)
            });

        attributes
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[traced_test]
    #[test]
    fn test_padding() {
        AttributeParser::parse_attributes("padding:1");
    }
}
