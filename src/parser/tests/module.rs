use super::parse;

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

#[test]
fn module_value() {
    parse(r#"{<size:1> text<size:sub!{topic:"text-size"}>("hello")}"#);
}

/// Test that the parser rejects module argument names starting with underscore, which are reserved
/// for internal use (ie element Attribute parser appending module arguments)
///
/// This test should panic.
#[test]
#[should_panic]
fn module_reserved_argument() {
    parse(r#"{text<size:sub!{_topic:"test"}>("hello")}"#);
}
