mod conversion;
mod data;
mod error;
mod message;
mod parser;

pub use conversion::theme::SnowcapTheme;
pub use error::*;
pub use message::Message;
pub use parser::Attribute;
pub use parser::MarkupTree;
pub use parser::SnowcapParser;
pub use parser::Value;

#[derive(Debug)]
pub struct Snowcap<AppMessage> {
    root: MarkupTree<AppMessage>,
}

impl<AppMessage> Snowcap<AppMessage> {
    pub fn root(&self) -> &MarkupTree<AppMessage> {
        &self.root
    }
}
