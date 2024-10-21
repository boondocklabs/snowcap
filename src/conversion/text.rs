use iced::widget::text::IntoFragment;

use crate::parser::Value;

/// Converts a [`Value`] reference into a [`Cow<'a, str>`].
///
/// This conversion is used to represent different `Value` variants as string data.
///
/// The possible variants and their conversions:
/// - `Value::String(s)` returns the cloned string.
/// - `Value::Number(n)` is formatted as a string.
/// - `Value::Boolean(b)` is formatted as a string.
/// - `Value::Null` is formatted as `"null"`.
/// - `Value::DataSource`:
///   - If the `DataProvider` is a `File` and its data is `Text`, returns the text data.
///   - Otherwise, it returns `"Unsupported DataProvider"` or `"Unknown DataType"`.
///
/// # Examples
///
/// ```rust
/// use snowcap::Value;
/// let value = Value::Number(42.0);
/// let cow: std::borrow::Cow<'_, str> = (&value).into();
/// assert_eq!(cow, "42");
/// ```
impl<'a> Into<std::borrow::Cow<'a, str>> for &Value {
    fn into(self) -> std::borrow::Cow<'a, str> {
        match self {
            Value::String(s) => s.clone().into(),
            Value::Number(n) => format!("{n}").into(),
            Value::Boolean(b) => format!("{b}").into(),
            Value::Dynamic { data, .. } => match &*data {
                Some(data) => match &**data {
                    crate::data::DataType::Text(text) => text.clone().into(),
                    _ => "Unknown DataType".into(),
                },
                None => format!("No Data Loaded").into(),
            },
            Value::Array(_value) => todo!(),
        }
    }
}

impl<'a> IntoFragment<'a> for &Value {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        self.into()
    }
}
