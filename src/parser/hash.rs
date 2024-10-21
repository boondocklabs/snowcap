use std::hash::Hash;

use tracing::warn;

use crate::data::DataType;

use super::Value;

impl Hash for DataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DataType::Null => {}
            DataType::Image(handle) => handle.id().hash(state),
            DataType::Svg(handle) => handle.id().hash(state),
            DataType::QrCode(_data) => {
                warn!("Can't hash QRCode");
            }
            DataType::Markdown(_markdown_items) => {
                warn!("Can't hash Markdown");
                /*
                for item in &**markdown_items.inner() {
                }
                */
            }
            DataType::Text(text) => text.hash(state),
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Value::String(s) => s.hash(state),
            Value::Number(num) => state.write(&num.to_ne_bytes()),
            Value::Boolean(b) => b.hash(state),
            Value::Array(vec) => vec.hash(state),
            Value::Dynamic { data, provider: _ } => {
                data.hash(state);
                // TODO: Hash provider state
            }
        }
    }
}
