# Snowcap

Early stage markup for [iced](https://iced.rs) using [pest](https://pest.rs)

There is a simple viewer in bin/snowcap-viewer.rs with hot reloading.
To run it, use `cargo run samples/test.iced` from the root of the project.

Here's an example of how the `test.iced` file renders.

<img width="1285" alt="Screenshot 2024-09-27 at 12 08 07 AM" src="https://github.com/user-attachments/assets/b751943f-e08a-4634-8223-49cf222f2772">

## Grammar

The grammar is specified in [snowcap.pest](src/snowcap.pest) and  an example layout is in [test.iced](samples/test.iced).


|Iced Element   | Snowcap Syntax |
|---------------|---------------------|
| Container     | `{<attr:val,...> ...}`|
| Row		| `-<attr:val,...>[ element, ...]`
| Column	| `\|<attr:val,...>[ element, ...]`
| Stack   | `^<attr:val,...>[ element, ...]`
| Rule (horiz)  | `rule-horizontal<height:2>()`
| Rule (vert)   | `rule-vertical<width:2>()`
| Text          | `text<attr:val,...>("Content")`
| Button        | `button<attr:val,...>(element)`
| Toggler       | `toggler<attr:val,...>(element)`
| QRCode	| `qrcode<cell-size:10>(qr!("https://iced.rs"))`
| Markdown      | `markdown(file!("README.md"))`
| Image         | `image(file!("samples/ferris.png"))`

For example, creating a container with a column would look like

```
{<width:"fill", align-x:"center">
	|<align:"center">[
		text<size:19>("Hello"),
		text<size:24>("Snowcap")
	]
}
```

<img width="537" alt="Screenshot 2024-09-25 at 8 36 26 PM" src="https://github.com/user-attachments/assets/db014468-8e9a-46c7-b7ee-d8e418077ce6">


