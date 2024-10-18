use iced::Color;
use pest::Parser;
use pest_derive::Parser;
use tracing::debug;

use super::ParseError;

#[derive(Parser)]
#[grammar = "parser/color.pest"]
pub struct ColorParser;

impl ColorParser {
    pub fn parse_str(data: &str) -> Result<iced::Color, ParseError> {
        debug!("Parsing color string {data}");
        let pairs = ColorParser::parse(Rule::color, data)?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::color_hex => {
                    return Color::parse(pair.as_str())
                        .ok_or(ParseError::InvalidColor(pair.as_str().to_string()));
                }
                Rule::color_rgba => {
                    let mut inner = pair.into_inner();
                    let red: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;
                    let green: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;
                    let blue: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;
                    let alpha: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;

                    return Ok(Color::from_rgba(red, green, blue, alpha));
                }
                Rule::color_rgb => {
                    let mut inner = pair.into_inner();
                    let red: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;
                    let green: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;
                    let blue: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;

                    return Ok(Color::from_rgb(red, green, blue));
                }
                Rule::color_rgb8 => {
                    let mut inner = pair.into_inner();
                    let red: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;
                    let green: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;
                    let blue: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;

                    return Ok(Color::from_rgb8(red, green, blue));
                }
                Rule::color_rgba8 => {
                    let mut inner = pair.into_inner();
                    let red: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;
                    let green: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;
                    let blue: u8 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Integer)?;
                    let alpha: f32 = inner
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .map_err(ParseError::Float)?;

                    return Ok(Color::from_rgba8(red, green, blue, alpha));
                }
                _ => continue,
            }
        }

        Ok(iced::Color::BLACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use iced::Color;
    use tracing_test::traced_test; // Import the Color type from iced

    // Use relative_eq to test floating point values
    // are within epsilon
    fn color_eq(a: Color, b: Color) {
        assert_relative_eq!(a.r, b.r, max_relative = 0.01);
        assert_relative_eq!(a.g, b.g, max_relative = 0.01);
        assert_relative_eq!(a.b, b.b, max_relative = 0.01);
        assert_relative_eq!(a.a, b.a, max_relative = 0.01);
    }

    #[traced_test]
    #[test]
    fn test_parse_hex_color_invalid() {
        let result = ColorParser::parse_str("#fffff");
        assert!(
            result.is_err(),
            "Expected error parsing invalid hex color string"
        );
    }

    #[traced_test]
    #[test]
    fn test_parse_hex_color_rrggbbaa() {
        let result = ColorParser::parse_str("#FF5733");
        assert!(result.is_ok(), "Expected successful parsing of HEX color.");

        if let Ok(color) = result {
            color_eq(color, Color::from_rgb(1.0, 0.34, 0.2))
        }
    }

    #[traced_test]
    #[test]
    fn test_parse_hex_color_rgb() {
        let result = ColorParser::parse_str("#FFF");
        assert!(result.is_ok(), "Expected successful parsing of HEX color.");

        if let Ok(color) = result {
            color_eq(color, Color::from_rgb(1.0, 1.0, 1.0))
        }
    }

    #[traced_test]
    #[test]
    fn test_parse_hex_color_alpha() {
        let result = ColorParser::parse_str("#FF5733FF");
        assert!(result.is_ok(), "Expected successful parsing of HEX color.");

        if let Ok(color) = result {
            color_eq(color, Color::from_rgba(1.0, 0.34, 0.2, 1.0))
        }
    }

    #[traced_test]
    #[test]
    fn test_parse_rgb_color() {
        let result = ColorParser::parse_str("1.0, 0.34, 0.2");
        assert!(result.is_ok(), "expected successful parsing of rgb color.");

        // assuming todo!() is implemented, match the expected result:
        if let Ok(color) = result {
            assert_eq!(
                color,
                Color::from_rgb(1.0, 0.34, 0.2),
                "expected correct rgb color."
            );
        }
    }

    #[traced_test]
    #[test]
    fn test_parse_rgb8_color() {
        let result = ColorParser::parse_str("255, 127, 255");
        assert!(result.is_ok(), "expected successful parsing of rgb color.");

        // assuming todo!() is implemented, match the expected result:
        if let Ok(color) = result {
            color_eq(color, Color::from_rgb(1.0, 0.5, 1.0));
        }
    }

    #[traced_test]
    #[test]
    fn test_parse_rgba8_color() {
        let result = ColorParser::parse_str("255, 127, 255, 0.5");
        assert!(result.is_ok(), "expected successful parsing of rgb color.");

        // assuming todo!() is implemented, match the expected result:
        if let Ok(color) = result {
            color_eq(color, Color::from_rgba(1.0, 0.5, 1.0, 0.5));
        }
    }

    #[test]
    fn test_parse_rgba_color() {
        let result = ColorParser::parse_str("1.0, 0.34, 0.2, 0.5");
        assert!(result.is_ok(), "Expected successful parsing of RGBA color.");

        // Assuming todo!() is implemented, match the expected result:
        if let Ok(color) = result {
            assert_eq!(
                color,
                Color::from_rgba(1.0, 0.34, 0.2, 0.5),
                "Expected correct RGBA color."
            );
        }
    }

    #[test]
    fn test_invalid_color_format() {
        let result = ColorParser::parse_str("invalid-color");
        assert!(
            result.is_err(),
            "Expected parsing to fail for invalid color format."
        );
    }
}
