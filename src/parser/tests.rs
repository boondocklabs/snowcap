use arbutus::Tree;
use colored::Colorize;

mod module;

use crate::{Message, NodeRef, SnowcapParser};

type M = Message;

fn parse(snow: &str) -> Tree<NodeRef> {
    let tree = SnowcapParser::<M>::parse_memory(snow);

    match &tree {
        Ok(tree) => println!("\n{}\n{}", "Parsed Tree:".bright_yellow(), tree.root()),
        Err(e) => println!("{:#?}\n\n{}", e, e),
    }

    assert!(tree.is_ok());
    tree.unwrap()
}

#[test]
fn text() {
    parse(r#"{text("foo")}"#);
}

#[test]
fn row() {
    parse(r#"{row[text("a"), text("b")]}"#);
}

#[test]
fn col() {
    parse(r#"{col[text("a"), text("b")]}"#);
}
