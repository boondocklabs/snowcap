use crate::{error::ConversionError, Value};
use iced::Theme;

/// A wrapper around the `Theme` enum that provides additional functionality,
/// such as converting a string representation of a theme into its corresponding
/// `Theme` variant.
pub struct SnowcapTheme(pub Theme);

impl SnowcapTheme {
    /// Returns a reference to the inner `Theme`.
    ///
    /// # Returns
    ///
    /// A reference to the `Theme` enum stored in this `SnowcapTheme` wrapper.
    ///
    /// # Examples
    ///
    /// ```
    /// use iced::Theme;
    /// use snowcap::SnowcapTheme;
    /// let theme = SnowcapTheme(Theme::Light);
    /// assert_eq!(theme.theme(), &Theme::Light);
    /// ```
    pub fn theme(&self) -> &Theme {
        &self.0
    }
}

impl TryFrom<&str> for SnowcapTheme {
    type Error = ConversionError;

    /// Attempts to convert a string into a `SnowcapTheme`.
    ///
    /// This method is case-insensitive and will convert a given string to
    /// its corresponding `Theme` variant. If the string doesn't match any of
    /// the predefined theme names, an error will be returned.
    ///
    /// # Parameters
    ///
    /// * `theme_name`: A string slice that represents the name of the theme.
    ///
    /// # Returns
    ///
    /// If the string matches a valid theme name, a `SnowcapTheme` is returned
    /// wrapped in a `Result`. If no match is found, a `ConversionError::Unknown`
    /// is returned.
    ///
    /// # Errors
    ///
    /// Returns a `ConversionError::Unknown` if the input string does not match
    /// any valid theme names.
    ///
    /// # Examples
    ///
    /// ```
    /// use iced::Theme;
    /// use snowcap::{SnowcapTheme,ConversionError};
    /// let theme: Result<SnowcapTheme, ConversionError> = SnowcapTheme::try_from("dracula");
    /// assert_eq!(theme.unwrap().theme(), &Theme::Dracula);
    ///
    /// let invalid_theme = SnowcapTheme::try_from("unknown_theme");
    /// assert!(invalid_theme.is_err());
    /// ```

    fn try_from(theme_name: &str) -> Result<Self, ConversionError> {
        let theme = match theme_name.to_lowercase().as_str() {
            "light" => SnowcapTheme(Theme::Light),
            "dark" => SnowcapTheme(Theme::Dark),
            "dracula" => SnowcapTheme(Theme::Dracula),
            "nord" => SnowcapTheme(Theme::Nord),
            "solarizedlight" => SnowcapTheme(Theme::SolarizedLight),
            "solarizeddark" => SnowcapTheme(Theme::SolarizedDark),
            "gruvboxlight" => SnowcapTheme(Theme::GruvboxLight),
            "gruvboxdark" => SnowcapTheme(Theme::GruvboxDark),
            "catppuccinlatte" => SnowcapTheme(Theme::CatppuccinLatte),
            "catppuccinfrappe" => SnowcapTheme(Theme::CatppuccinFrappe),
            "catppuccinmacchiato" => SnowcapTheme(Theme::CatppuccinMacchiato),
            "catppuccinmocha" => SnowcapTheme(Theme::CatppuccinMocha),
            "tokyonight" => SnowcapTheme(Theme::TokyoNight),
            "tokyonightstorm" => SnowcapTheme(Theme::TokyoNightStorm),
            "tokyonightlight" => SnowcapTheme(Theme::TokyoNightLight),
            "kanagawawave" => SnowcapTheme(Theme::KanagawaWave),
            "kanagawadragon" => SnowcapTheme(Theme::KanagawaDragon),
            "kanagawalotus" => SnowcapTheme(Theme::KanagawaLotus),
            "moonfly" => SnowcapTheme(Theme::Moonfly),
            "nightfly" => SnowcapTheme(Theme::Nightfly),
            "oxocarbon" => SnowcapTheme(Theme::Oxocarbon),
            "ferra" => SnowcapTheme(Theme::Ferra),
            _ => {
                return Err(ConversionError::Unknown(format!(
                    "Unknown theme '{theme_name}'"
                )))
            }
        };

        Ok(theme)
    }
}

impl TryInto<Theme> for &Value {
    type Error = ConversionError;

    /// Attempts to convert a `Value` reference into a `Theme`.
    ///
    /// This implementation first tries to convert the `Value` into a string
    /// representation of the theme name. It then uses the `SnowcapTheme` type
    /// to convert that string into the appropriate `Theme` variant.
    ///
    /// # Returns
    ///
    /// If successful, returns a `Theme` variant. If the conversion fails due
    /// to an invalid `Value` or an unrecognized theme name, a `ConversionError`
    /// is returned.
    ///
    /// # Errors
    ///
    /// This function returns a `ConversionError` in two cases:
    /// 1. If the `Value` cannot be converted into a string.
    /// 2. If the resulting string is not a recognized theme name.
    ///
    /// # Examples
    ///
    /// ```
    /// use iced::Theme;
    /// use snowcap::{Value,ConversionError};
    /// let value = Value::String(String::from("dracula"));
    /// let theme: Result<Theme, ConversionError> = (&value).try_into();
    /// assert_eq!(theme.unwrap(), Theme::Dracula);
    ///
    /// let invalid_value = Value::Number(42.into());
    /// let result: Result<Theme, ConversionError> = (&invalid_value).try_into();
    /// assert!(result.is_err());
    /// ```

    fn try_into(self) -> Result<Theme, Self::Error> {
        let theme_name: &String = self.try_into()?;

        let wrapped_theme = SnowcapTheme::try_from(theme_name.as_str())?;

        // Return the inner iced::Theme
        Ok(wrapped_theme.0)
    }
}

/*
impl TryInto<Theme> for &Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<Theme, Self::Error> {
        (&*self.value()).try_into()
    }
}

impl TryInto<Theme> for Attribute {
    type Error = ConversionError;

    fn try_into(self) -> Result<Theme, Self::Error> {
        (&*self.value()).try_into()
    }
}
*/

#[cfg(test)]
mod test {

    use super::SnowcapTheme;

    #[test]
    pub fn from_string() {
        let _theme = SnowcapTheme::try_from("Light").unwrap().theme();
    }
}
