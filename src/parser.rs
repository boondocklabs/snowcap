use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::debug;

use crate::data::{DataProvider, FileProvider, QRDataProvider};
use crate::error::ParseError;
use crate::Snowcap;

#[derive(Parser)]
#[grammar = "snowcap.pest"]
pub struct SnowcapParser;

impl SnowcapParser {
    pub fn parse_file(file: &str) -> Result<Snowcap, crate::Error> {
        let markup = SnowcapParser::parse(Rule::markup, file)?.next().unwrap();
        let root = SnowcapParser::parse_pair(markup);
        Ok(Snowcap { root })
    }

    fn parse_attributes(pairs: Pairs<Rule>) -> Attributes {
        debug!("[Parsing Attributes]");

        pairs
            .map(|pair| {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let value = Self::parse_value(
                    inner
                        .next()
                        .expect("Expected attributed value following label"),
                )
                .unwrap();
                Attribute { name, value }
            })
            .collect()
    }

    fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
        match pair.as_rule() {
            Rule::null => Ok(Value::Null),
            Rule::number => Ok(Value::Number(pair.as_str().parse().unwrap())),
            Rule::string => Ok(Value::String(pair.into_inner().as_str().into())),
            Rule::boolean => Ok(Value::Boolean(pair.as_str().parse().unwrap())),
            Rule::data_source => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let value = inner
                    .next()
                    .expect("Expected data source value")
                    .into_inner()
                    .as_str()
                    .to_string();

                let provider = match name.as_str() {
                    "qr" => {
                        let qr_data = iced::widget::qr_code::Data::new(&value).unwrap();
                        DataProvider::QrCode(QRDataProvider::new(qr_data))
                    }
                    "file" => {
                        let mut provider = FileProvider::new(&value);
                        if value.ends_with(".md") {
                            provider.load_markdown().unwrap();
                        } else {
                            provider.load_text().unwrap();
                        }
                        DataProvider::File(provider)
                    }
                    _ => DataProvider::None,
                };

                Ok(Value::DataSource {
                    name,
                    value,
                    provider,
                })
            }
            _ => Err(ParseError::Unhandled(format!(
                "AttributeValue {:?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_container(pair: Pair<Rule>) -> Result<MarkupType, ParseError> {
        debug!("[Parsing Container]");

        let inner = pair.into_inner();

        let mut content: MarkupType = MarkupType::None;
        let mut attr = Attributes::default();

        for pair in inner {
            match pair.as_rule() {
                Rule::row | Rule::column | Rule::element | Rule::stack => {
                    content = Self::parse_pair(pair);
                }
                Rule::attributes => {
                    attr = Self::parse_attributes(pair.into_inner());
                    debug!("{attr:?}");
                }
                _ => return Err(ParseError::UnsupportedRule(format!("{:?}", pair.as_rule()))),
            };
        }

        Ok(MarkupType::Container {
            content: Box::new(content),
            attrs: attr,
        })
    }

    fn parse_row(pair: Pair<Rule>) -> Result<MarkupType, ParseError> {
        let mut contents = Vec::new();
        let mut attrs = Attributes::default();

        for pair in pair.into_inner() {
            if let Rule::attributes = pair.as_rule() {
                attrs = Self::parse_attributes(pair.into_inner());
                continue;
            }

            contents.push(SnowcapParser::parse_pair(pair))
        }

        Ok(MarkupType::Row { attrs, contents })
    }

    fn parse_column(pair: Pair<Rule>) -> Result<MarkupType, ParseError> {
        let mut contents = Vec::new();
        let mut attrs = Attributes::default();

        for pair in pair.into_inner() {
            if let Rule::attributes = pair.as_rule() {
                attrs = Self::parse_attributes(pair.into_inner());
                continue;
            }

            contents.push(SnowcapParser::parse_pair(pair))
        }

        Ok(MarkupType::Column { attrs, contents })
    }

    fn parse_stack(pair: Pair<Rule>) -> Result<MarkupType, ParseError> {
        let mut contents = Vec::new();
        let mut attrs = Attributes::default();

        for pair in pair.into_inner() {
            if let Rule::attributes = pair.as_rule() {
                attrs = Self::parse_attributes(pair.into_inner());
                continue;
            }

            contents.push(SnowcapParser::parse_pair(pair))
        }

        Ok(MarkupType::Stack { attrs, contents })
    }

    fn parse_pair(pair: Pair<Rule>) -> MarkupType {
        match pair.as_rule() {
            Rule::container => Self::parse_container(pair).unwrap(),
            Rule::row => Self::parse_row(pair).unwrap(),
            Rule::column => Self::parse_column(pair).unwrap(),
            Rule::stack => Self::parse_stack(pair).unwrap(),
            Rule::element => {
                debug!("Element {pair:?}");
                let mut inner = pair.into_inner();
                let label = inner.next().unwrap().as_str().to_string();

                let mut attr = Attributes::default();
                let mut value = MarkupType::None;

                for pair in inner {
                    match pair.as_rule() {
                        Rule::attributes => {
                            attr = Self::parse_attributes(pair.into_inner());
                        }

                        Rule::element_value => {
                            let val = Self::parse_value(pair.into_inner().next().unwrap());
                            value = MarkupType::Value(val.unwrap());
                        }
                        Rule::element => {
                            value = Self::parse_pair(pair);
                        }
                        _ => {}
                    }
                }

                MarkupType::Element {
                    name: label,
                    attrs: attr,
                    content: Box::new(value),
                }
            }
            Rule::element_value => SnowcapParser::parse_pair(pair.into_inner().last().unwrap()),
            Rule::label => MarkupType::Label(pair.into_inner().next().unwrap().as_str().into()),
            _ => panic!("Unhandled {pair:?}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    pub fn get(&self, name: &str) -> Option<&Attribute> {
        for attr in &self.0 {
            if attr.name.as_str() == name {
                return Some(attr);
            }
        }
        None
    }
}

impl IntoIterator for Attributes {
    type Item = Attribute;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = &'a Attribute;

    type IntoIter = core::slice::Iter<'a, Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        let v = &self.0;
        v.iter()
    }
}

impl FromIterator<Attribute> for Attributes {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        Self(c)
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    DataSource {
        name: String,
        value: String,
        //provider: Box<dyn DataProvider + 'static>,
        provider: DataProvider,
    },
}

/*
impl Clone for Box<dyn DataProvider> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl std::fmt::Debug for dyn DataProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn DataProvider")
    }
}
*/

#[derive(Debug)]
pub enum MarkupType {
    None,
    Container {
        content: Box<MarkupType>,
        attrs: Attributes,
    },
    Element {
        name: String,
        attrs: Attributes,
        content: Box<MarkupType>,
    },
    Row {
        attrs: Attributes,
        contents: Vec<MarkupType>,
    },
    Column {
        attrs: Attributes,
        contents: Vec<MarkupType>,
    },
    Stack {
        attrs: Attributes,
        contents: Vec<MarkupType>,
    },
    Label(String),
    Value(Value),
}

#[cfg(test)]
mod tests {}
