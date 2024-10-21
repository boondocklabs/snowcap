use super::{Value, ValueKind};
use crate::data::DataType;
use tracing::warn;

impl std::hash::Hash for DataType {
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

impl std::hash::Hash for ValueKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            ValueKind::String(s) => s.hash(state),
            ValueKind::Float(num) => state.write(&num.to_ne_bytes()),
            ValueKind::Integer(num) => state.write(&num.to_ne_bytes()),
            ValueKind::Boolean(b) => b.hash(state),
            ValueKind::Array(vec) => vec.hash(state),
            ValueKind::Dynamic { data: _, provider } => {
                //data.hash(state);

                if let Some(provider) = provider {
                    let provider = provider.lock();
                    provider.hash_source(state);
                }
            }
        }
    }
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}
