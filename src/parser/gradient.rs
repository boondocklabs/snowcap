use iced::{gradient::Linear, Gradient};
use pest::Parser;
use pest_derive::Parser;
use tracing::debug;

use crate::parser::color::ColorParser;

use super::ParseError;

#[derive(Parser)]
#[grammar = "parser/gradient.pest"]
pub struct GradientParser;

impl GradientParser {
    pub fn parse_str(data: &str) -> Result<Gradient, ParseError> {
        debug!("Parsing gradient string {data}");
        let pairs = GradientParser::parse(Rule::gradient, data)?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::gradient => {
                    let mut inner = pair.into_inner();
                    let rad: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;

                    debug!("Gradient rad {rad}");

                    let mut linear = Linear::new(rad);

                    let stops = inner.next().unwrap().into_inner();
                    for stop in stops {
                        let mut inner = stop.into_inner();
                        let color_str = inner.next().unwrap().as_str();
                        let offset: f32 = inner
                            .next()
                            .unwrap()
                            .as_str()
                            .parse()
                            .map_err(ParseError::Float)?;

                        let color = ColorParser::parse_str(color_str)?;

                        debug!("Stop offset={} color={:?}", offset, color);

                        linear = linear.add_stop(offset, color);
                    }

                    return Ok(Gradient::Linear(linear));
                }
                _ => continue,
            }
        }

        Ok(Gradient::Linear(Linear::new(1.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_parse_gradient() {
        let result = GradientParser::parse_str("1.57, [10,20,30@0.2, 30,50,120@0.8]");
        assert!(result.is_ok(), "Expected successful parsing of gradient.");

        if let Ok(gradient) = result {
            tracing::info!("Got gradient {gradient:#?}");
        }
    }
}
