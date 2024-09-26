# Snowcap

Early stage markup for [iced](iced.rs) using [pest](pest.rs)

There is a simple viewer in bin/ with hot reloading.
To run it, use `cargo run test.iced` from the root of the project.

## Grammar

The grammar is specified in [snowcap.pest](src/snowcap.pest) and  an example layout is in [test.iced](test.iced).


|Iced Element   | Snowcap Syntax |
|---------------|---------------------|
| Container     | `{<attr:val,...> ...}`|
| Row			| `-<attr:val,...>[ element, ...]`
| Column		| `|<attr:val,...>[ element, ...]`
| Text          | `text<attr:val,...>("Content")`
| Button        | `button<attr:val,...>(element)`

For example, creating a container with a column would look like

```
{<width:"fill", align-x:"center">
	|<align:"center">[
		text<size:19>("Hello"),
		text<size:24>("Snowcap")
	]
}
```
