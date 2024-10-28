#[cfg(test)]
mod tests {
    use arbutus::TreeDiff;
    use colored::Colorize as _;
    use tracing_test::traced_test;

    use crate::{IndexedTree, Message, SnowcapParser};
    fn print_trees<M>(a: &IndexedTree<M>, b: &IndexedTree<M>)
    where
        M: std::fmt::Debug,
    {
        println!(
            "{}\n{}",
            "----========[  Dest Tree  ]========----:".bright_purple(),
            a.root()
        );
        println!(
            "{}\n{}",
            "----========[ Source Tree ]========----:".bright_purple(),
            b.root()
        );
    }

    fn tree_eq<M>(a: &IndexedTree<M>, b: &IndexedTree<M>)
    where
        M: std::fmt::Debug,
    {
        print_trees(a, b);

        if a == b {
            println!("{}", "Trees are identical".bright_green())
        } else {
            println!("{}", "Trees are different".bright_red())
        }

        assert_eq!(a, b);
    }

    #[traced_test]
    #[test]
    fn identical() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        assert_eq!(patches.len(), 0);
    }

    #[traced_test]
    #[test]
    fn different() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{text("Hello")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);

        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("Hello")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);

        let mut a = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("Hello"), col[text("Test")]]}"#,
        )
        .unwrap()
        .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text("World")}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);

        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{-[text("A"), text("B")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{-[text("A"), text("B"), text("C")]}"#,
        )
        .unwrap()
        .index();

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn modify_attrs() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("A")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:20>("A")}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);

        assert_eq!(patches.len(), 1);

        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn modify_text() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("A")}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{text<size:10>("B")}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn add_child() {
        // Test adding a node to a row
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{row[text("a")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{row[text("a"), text("b")]}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);

        // Test adding a node to a row
        let mut a = SnowcapParser::<Message<String>>::parse_memory(r#"{row[text("a")]}"#)
            .unwrap()
            .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{row[text("a"), text("b"), text("c")]}"#,
        )
        .unwrap()
        .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn add_child_middle() {
        // Test adding a node to a row
        let mut a = SnowcapParser::<Message<String>>::parse_memory(
            r#"{row[text("a"), text<size:10>("c")]}"#,
        )
        .unwrap()
        .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{row[text("a"), text("b"), text<size:20>("c")]}"#,
        )
        .unwrap()
        .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn modify_depth() {
        // Test modifying a child with a new subtree
        let mut a = SnowcapParser::<Message<String>>::parse_memory(
            r#"{row[text("a"), text<size:10>("c")]}"#,
        )
        .unwrap()
        .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"{row[col[text("a")], text<size:10>("c")]}"#,
        )
        .unwrap()
        .index();

        print_trees(&a, &b);

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn delete_child() {
        // Test adding a node to a row
        let mut a =
            SnowcapParser::<Message<String>>::parse_memory(r#"{row[text("a"), text("b")]}"#)
                .unwrap()
                .index();
        let b = SnowcapParser::<Message<String>>::parse_memory(r#"{row[text("a")]}"#)
            .unwrap()
            .index();
        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn space_image() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(
            r#"
        {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
            // Left column
            col[
                //text("inserting"),
                svg(file!("samples/coder.svg")),
                pick-list#foo<selected:"abc">(["abc", "bar"]),
                pick-list#bar<selected:"bar">(["baz", "bar"]),
                text<size:24>("I'm some text"),
                text<size:10>("More text in a Column"),
                text(url!("http://icanhazip.com")),
                image(url!("https://picsum.photos/200/300")),
                space<size:10>(),
                {<height:fill, align-y:center>
                    text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                }
            ]
        }
        "#,
        ).unwrap().index();

        let b = SnowcapParser::<Message<String>>::parse_memory(
            r#"
        {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
            // Left column
            col[
                text("inserting"),
                svg(file!("samples/coder.svg")),
                pick-list#foo<selected:"abc">(["abc", "bar"]),
                pick-list#bar<selected:"bar">(["baz", "bar"]),
                text<size:24>("I'm some text"),
                text<size:10>("More text in a Column"),
                text(url!("http://icanhazip.com")),
                image(url!("https://picsum.photos/200/300")),
                space<size:10>(),
                {<height:fill, align-y:center>
                    text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                }
            ]
        }
        "#,
        ).unwrap().index();

        println!("{}", a.root());

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);
        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn add_column() {
        // Test adding a node to a row
        let mut a =
            SnowcapParser::<Message<String>>::parse_memory(
                r#"
                // Create a top level container
                {<bg:gradient(0.8,[#202030@0.0, #404045@0.3, #323030@1.0]), text-color:#ffffff>

                    // Create a column for the top bar, and bottom content
                    col<padding:5>[
                        // Top bar container with a fixed height, and filling the width
                        {<align-x:center, align-y:center, padding:10,10,10,10, height:100, width:fill, border:color(#a0a0a0),width(1.0),radius(6.0), bg:color(#c0c0a010)>
                            row<align:center, spacing:10>[
                                image#ferris(file!("samples/ferris.png")),
                                text#title<size:40, text-color:#000000, wrapping:none, shaping:advanced>("Snowcap Viewer")
                            ]
                        },

                        // Bottom container
                        {
                            row<height:fill, padding:top(10), spacing:10>[
                                // Left column container
                                {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
                                    // Left column
                                    col[
                                        //text("inserting"),
                                        svg(file!("samples/coder.svg")),
                                        pick-list#foo<selected:"abc">(["abc", "bar"]),
                                        pick-list#bar<selected:"bar">(["baz", "bar"]),
                                        text<size:24>("I'm some text"),
                                        text<size:10>("More text in a Column"),
                                        text(url!("http://icanhazip.com")),
                                        image(url!("https://picsum.photos/200/300")),
                                        space<size:10>(),
                                        {<height:fill, align-y:center>
                                            text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                                        }
                                    ]
                                },

                                // Middle Column container
                                {<width:fill, height:fill, align-x:center, align-y:top, padding:10.0, border:color(#a0a0a0), width(1), radius(10)>
                                    // Middle column (shorthand |)
                                    |<align:center>[
                                        markdown(file!("README.md")),
                                        qr-code<cell-size:10>(qr!("https://iced.rs")),
                                        button#my-button(text<size:20>("Button")),
                                        toggler#toggle-a<toggled:false, label:"Foo", size:20>(),
                                        toggler#toggle-b<toggled:false, label:"Bar", size:30>(),
                                        toggler<toggled:false, label:"Baz", size:40>()
                                    ]
                                },

                                // Right Column
                                {<width:200, height:fill, align-x:left, padding:10.0, border:color(#a0a0a0), width(1), radius(10)>
                                    |[
                                        text<size:30>("Ipsum"),
                                        text(url!("http://corporatelorem.kovah.de/api/3?format=text"))
                                    ]
                                }
                            ]
                        }
                    ]
                }
                "#
            )
                .unwrap()
                .index();

        let b =
            SnowcapParser::<Message<String>>::parse_memory(
                r#"
                // Create a top level container
                {<bg:gradient(0.8,[#202030@0.0, #404045@0.3, #323030@1.0]), text-color:#ffffff>

                    // Create a column for the top bar, and bottom content
                    col<padding:5>[
                        // Top bar container with a fixed height, and filling the width
                        {<align-x:center, align-y:center, padding:10,10,10,10, height:100, width:fill, border:color(#a0a0a0),width(1.0),radius(6.0), bg:color(#c0c0a010)>
                            row<align:center, spacing:10>[
                                image#ferris(file!("samples/ferris.png")),
                                text#title<size:40, text-color:#000000, wrapping:none, shaping:advanced>("Snowcap Viewer")
                            ]
                        },

                        // Bottom container
                        {
                            row<height:fill, padding:top(10), spacing:10>[
                                // Left column container
                                {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
                                    // Left column
                                    col[
                                        text("inserting"),
                                        svg(file!("samples/coder.svg")),
                                        pick-list#foo<selected:"abc">(["abc", "bar"]),
                                        pick-list#bar<selected:"bar">(["baz", "bar"]),
                                        text<size:24>("I'm some text"),
                                        text<size:10>("More text in a Column"),
                                        text(url!("http://icanhazip.com")),
                                        image(url!("https://picsum.photos/200/300")),
                                        space<size:10>(),
                                        {<height:fill, align-y:center>
                                            text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                                        }
                                    ]
                                },

                                // Middle Column container
                                {<width:fill, height:fill, align-x:center, align-y:top, padding:10.0, border:color(#a0a0a0), width(1), radius(10)>
                                    // Middle column (shorthand |)
                                    |<align:center>[
                                        markdown(file!("README.md")),
                                        qr-code<cell-size:10>(qr!("https://iced.rs")),
                                        button#my-button(text<size:20>("Button")),
                                        toggler#toggle-a<toggled:false, label:"Foo", size:20>(),
                                        toggler#toggle-b<toggled:false, label:"Bar", size:30>(),
                                        toggler<toggled:false, label:"Baz", size:40>()
                                    ]
                                },

                                // Right Column
                                {<width:200, height:fill, align-x:left, padding:10.0, border:color(#a0a0a0), width(1), radius(10)>
                                    |[
                                        text<size:30>("Ipsum"),
                                        text(url!("http://corporatelorem.kovah.de/api/3?format=text"))
                                    ]
                                }
                            ]
                        }
                    ]
                }
                "#
            )
                .unwrap()
                .index();

        println!("{}", a.root());

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);

        tree_eq(&a, &b);
    }

    #[traced_test]
    #[test]
    fn add_row() {
        let mut a = SnowcapParser::<Message<String>>::parse_memory(
        r#"
            {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
                col[
                    text("foo"),
                    svg(file!("samples/coder.svg")),
                    text<size:24>("I'm some text")
                ]
            }
            "#).unwrap().index();

        let b = SnowcapParser::<Message<String>>::parse_memory(
        r#"
            {<width:200, height:fill, align-x:center, padding: 10.0, border:color(#0090a0),width(2),radius(10)>
                col[
                    row[text("foo"), text("bar")],
                    svg(file!("samples/coder.svg")),
                    text<size:24>("I'm some text")
                ]
            }
            "#).unwrap().index();

        print_trees(&a, &b);

        let mut diff = TreeDiff::new(a.root(), b.root());
        let patches = diff.diff();
        patches.patch_tree(&mut a);

        tree_eq(&a, &b);
    }
}
