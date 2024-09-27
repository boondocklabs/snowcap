mod conversion;
mod data;
mod error;
mod message;
mod parser;

pub use error::Error;
pub use message::Message;
pub use parser::MarkupTree;
pub use parser::SnowcapParser;

#[derive(Debug)]
pub struct Snowcap<AppMessage> {
    root: MarkupTree<AppMessage>,
}

impl<AppMessage> Snowcap<AppMessage> {
    pub fn root(&self) -> &MarkupTree<AppMessage> {
        &self.root
    }
}
