use super::ParserContext;
use crate::{
    data::{
        provider::{DynProvider, Provider},
        DataType,
    },
    ConversionError,
};
use iced::widget::text::IntoFragment;
use parking_lot::Mutex;
use std::{borrow::Borrow, fmt::Write, ops::Deref, sync::Arc};

#[derive(Clone, Debug)]
pub struct Value {
    context: Option<ParserContext>,
    inner: ValueKind,
}

impl Value {
    pub fn new(inner: ValueKind) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    pub fn new_string(val: String) -> Self {
        Self {
            inner: ValueKind::String(val),
            context: None,
        }
    }

    pub fn new_float(val: f64) -> Self {
        Self {
            inner: ValueKind::Float(val),
            context: None,
        }
    }

    pub fn new_integer(val: u64) -> Self {
        Self {
            inner: ValueKind::Integer(val),
            context: None,
        }
    }

    pub fn new_bool(val: bool) -> Self {
        Self {
            inner: ValueKind::Boolean(val),
            context: None,
        }
    }

    pub fn new_array(val: Vec<Self>) -> Self {
        Self {
            inner: ValueKind::Array(val),
            context: None,
        }
    }

    pub fn new_provider(provider: impl Provider<H = crate::SnowHasher> + 'static) -> Self {
        Self {
            inner: ValueKind::Dynamic {
                data: None,
                provider: Some(Arc::new(Mutex::new(provider))),
            },
            context: None,
        }
    }

    pub fn new_data(data: DataType) -> Self {
        Self {
            inner: ValueKind::Dynamic {
                data: Some(Arc::new(data)),
                provider: None,
            },
            context: None,
        }
    }

    pub fn with_context(mut self, context: ParserContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn inner(&self) -> &ValueKind {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut ValueKind {
        &mut self.inner
    }

    pub fn array(&self) -> Result<&Vec<Value>, ConversionError> {
        if let ValueKind::Array(array) = self.inner() {
            Ok(array)
        } else {
            Err(ConversionError::InvalidType(
                "expecting ValueKind::Array".into(),
            ))
        }
    }

    pub fn dynamic(&self) -> Result<&Option<Arc<DataType>>, ConversionError> {
        if let ValueKind::Dynamic { data, .. } = self.inner() {
            Ok(data)
        } else {
            Err(ConversionError::InvalidType(
                "expecting ValueKind::Dynamic".into(),
            ))
        }
    }
}

impl Deref for Value {
    type Target = ValueKind;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

#[derive(Clone, Debug)]
pub enum ValueKind {
    String(String),
    Float(f64),
    Integer(u64),
    Boolean(bool),
    Array(Vec<Value>),
    Dynamic {
        data: Option<Arc<DataType>>,
        provider: Option<Arc<Mutex<DynProvider>>>,
    },
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::String(s) => f.write_str(s),
            ValueKind::Float(num) => f.write_fmt(format_args!("{}f64", num)),
            ValueKind::Integer(num) => f.write_fmt(format_args!("{}u64", num)),
            ValueKind::Boolean(b) => f.write_fmt(format_args!("{}", b)),
            ValueKind::Array(vec) => {
                f.write_char('[')?;
                let mut iter = vec.iter().peekable();
                while let Some(val) = iter.next() {
                    write!(f, "{}", val)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                f.write_char(']')
            }
            ValueKind::Dynamic { provider, .. } => {
                if let Some(provider) = provider {
                    provider.lock().fmt(f)
                    //write!(f, "Dynamic {}", provider)
                } else {
                    write!(f, "Dynamic no provider")
                }
            }
        }
    }
}

impl Borrow<String> for Value {
    fn borrow(&self) -> &String {
        match &self.inner {
            ValueKind::String(s) => s,
            _ => panic!("Cannot borrow string for non-string typed value"),
        }
    }
}

impl<'a> IntoFragment<'a> for &ValueKind {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        self.into()
    }
}

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
impl<'a> Into<std::borrow::Cow<'a, str>> for &ValueKind {
    fn into(self) -> std::borrow::Cow<'a, str> {
        match self {
            ValueKind::String(s) => s.clone().into(),
            ValueKind::Float(n) => format!("{n}").into(),
            ValueKind::Integer(n) => format!("{n}").into(),
            ValueKind::Boolean(b) => format!("{b}").into(),
            ValueKind::Dynamic { data, .. } => match &*data {
                Some(data) => match &**data {
                    crate::data::DataType::Text(text) => text.clone().into(),
                    _ => "Unknown DataType".into(),
                },
                None => format!("No Data Loaded").into(),
            },
            ValueKind::Array(_value) => todo!(),
        }
    }
}
