mod conversion;
mod data;
mod error;
mod parser;

pub use error::Error;
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
