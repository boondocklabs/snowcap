// Create a top level container
{<bg:"gradient(0.8,[#202030@0.0, #404045@0.5, #323030@1.0])", text-color:"#ffffff">

    // Create a column for the top bar, and bottom content
    |[
        // Top bar container with a fixed height, and filling the width
        {<align-x:"center", align-y:"center", padding:0, height:100, width:"fill">
            row<align:"center", spacing:30>[
                image#ferris(file!("samples/ferris.png")),
                text#title<size:40>("Snowcap Viewer")
            ]
        },

        rule-horizontal<height:1>(),

        // Bottom container
        {
            -<height:"fill">[
                // Left column
                {<width:200, height:"fill", align-x:"center", padding: 10.0>
                    |<align:"center">[
                        svg(file!("samples/coder.svg")),
                        pick-list#foo<selected:"foo">(["foo", "bar"]),
                        pick-list#bar<selected:"bar">(["baz", "bar"]),
                        text<size:24>("I'm some text"),
                        text<size:10>("More text in a Column"),
                        text(url!("http://icanhazip.com")),
                        image(url!("https://picsum.photos/200/300")),
                        space<size:10>(),
                        {<height:"fill", align-y:"center">
                            text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                        }
                    ]
                },

                rule-vertical<width:2>(),

                // Middle Column
                {<width:"fill", height:"fill", align-x:"center", align-y:"top", padding:10.0>
                    |<align:"center">[
                        markdown(file!("README.md")),
                        qr-code<cell-size:10>(qr!("https://iced.rs")),
                        button#my-button(text<size:20>("Button")),
                        toggler#toggle-a<toggled:false, label:"Toggle Label Foo", size:30>(),
                        toggler#toggle-b<toggled:false, label:"Toggle Label Bar", size:30>(),
                        toggler<toggled:false, label:"Toggle Label Baz", size:30>()
                    ]
                },

                rule-vertical<width:2>(),

                // Right Column
                {<width:200, align-x:"center", padding:10.0>
                    |[
                        text<size:24>("Ipsum"),
                        text(file!("samples/filler.txt"))
                    ]
                }
            ]
        }
    ]
}
