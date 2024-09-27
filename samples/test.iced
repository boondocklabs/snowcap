{
    |[
        {<align-x:"center", align-y:"center", padding:0, height:100, width:"fill">
            row<align:"center">[
                text<size:60>("Snowcap Viewer")
            ]
        },

        rule-horizontal<height:1>(),

        -<height:"fill">[
            {<width:200, height:"fill", align-x:"center">
                |<align:"center">[
                    text<size:24>("I'm some text"),
                    text<size:10>("More text in a Column"),
                    space<size:10>(),
                    {<height:"fill", align-y:"center">
                        text<size:17>("Edit the test.iced file to see your changes hot reloaded")
                    }
                ]
            },

            rule-vertical<width:2>(),

            {<width:"fill", height:"fill", align-x:"center", align-y:"top">
                |<align:"center">[
                    markdown(file!("README.md")),
                    qr-code<cell-size:10>(qr!("https://iced.rs")),
                    button(text<size:20>("Text in a Button")),
                    toggler<toggled:true, size:30>()
                ]
            },

            rule-vertical<width:2>(),

            {<width:200, align-x:"center">
                |[
                    text<size:24>("Right!"),
                    text(file!("README.md"))
                ]
            }
        ]
    ]
}
