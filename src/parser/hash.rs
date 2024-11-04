use super::{Value, ValueData};
//use crate::data::DataType;

/*
impl std::hash::Hash for DataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

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
*/

impl std::hash::Hash for ValueData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            ValueData::String(s) => s.hash(state),
            ValueData::Float(num) => state.write(&num.to_ne_bytes()),
            ValueData::Integer(num) => state.write(&num.to_ne_bytes()),
            ValueData::Boolean(b) => b.hash(state),
            ValueData::Array(vec) => vec.hash(state),
            ValueData::None => {}
        }
    }
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the inner ValueKind
        (&**self).hash(state)
    }
}
