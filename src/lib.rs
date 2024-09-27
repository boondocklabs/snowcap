mod conversion;
mod data;
mod error;
mod message;
mod parser;

pub use error::Error;
pub use message::Message;
pub use parser::MarkupType;
pub use parser::SnowcapParser;

#[derive(Debug)]
pub struct Snowcap {
    root: MarkupType,
}

impl Snowcap {
    pub fn root(&self) -> &MarkupType {
        &self.root
    }
}
