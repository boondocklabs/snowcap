#[cfg(test)]
mod tests {
    use arbutus::Tree;
    use colored::Colorize;

    use crate::{Message, NodeRef, SnowcapParser};

    type M = Message<String>;

    fn parse(snow: &str) -> Tree<NodeRef<M>> {
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

    #[test]
    fn module_row() {
        parse(r#"{row[x!{}, y!{}, z!{}]}"#);
    }

    #[test]
    fn module_component() {
        parse(
            r#"
            {
                something!{path:"./test.txt"}
            }
            "#,
        );
    }

    /// Test parsing a module passed as widget content
    #[test]
    fn module_content() {
        parse(
            r#"
            {
                text(counter!{event: "button1", init:2})
            }
            "#,
        );
    }
}
