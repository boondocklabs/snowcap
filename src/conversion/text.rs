use iced::widget::text::IntoFragment;

use crate::{
    data::DataProvider,
    parser::{Attribute, Value},
    MarkupTree,
};

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
            Value::Null => format!("null").into(),
            Value::DataSource {
                name: _,
                value: _,
                provider,
            } => match provider {
                DataProvider::File(file_provider) => match file_provider.data() {
                    crate::data::DataType::Text(text) => text.clone().into(),
                    _ => "Unknown DataType".into(),
                },
                _ => "Unsupported DataProvider".into(),
            },
        }
    }
}

/// Converts a reference to a [`MarkupTree<AppMessage>`] into a text fragment.
///
/// This implementation extracts and converts the `MarkupTree::Value` variant into
/// an `iced::widget::text::Fragment`. If the `MarkupTree` is not a `Value` variant,
/// it returns a fragment indicating that `MarkupType::Value` was expected.
///
/// # Examples
///
/// ```rust
/// enum AppMessage {
///     None
/// };
/// use snowcap::{MarkupTree,Value};
/// use iced::advanced::text::IntoFragment;
/// let markup_tree = MarkupTree::<AppMessage>::Value(Value::String("Hello".to_string()));
/// let fragment: iced::widget::text::Fragment = (&markup_tree).into_fragment();
/// ```
impl<'a, AppMessage> IntoFragment<'a> for &MarkupTree<AppMessage> {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        match self {
            MarkupTree::Value(value) => value.into(),
            _ => "Expecting MarkupType::Value".into(),
        }
    }
}

/// Converts a reference to an [`Attribute`] into a text fragment.
///
/// This implementation extracts the value of the `Attribute` and converts it into
/// an `iced::widget::text::Fragment`.
///
/// # Examples
///
/// ```rust
/// use snowcap::{Attribute,Value};
/// use iced::advanced::text::IntoFragment;
/// let attr = Attribute { name: "test".into(), value: Value::String("Hello".to_string()) };
/// let fragment: iced::widget::text::Fragment = (&attr).into_fragment();
/// ```
impl<'a> IntoFragment<'a> for &Attribute {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        (&*self.value()).into()
    }
}
