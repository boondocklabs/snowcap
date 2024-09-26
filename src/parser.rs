use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::debug;

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
                );
                Attribute { name, value }
            })
            .collect()
    }

    fn parse_value(pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::null => Value::Null,
            Rule::number => Value::Number(pair.as_str().parse().unwrap()),
            Rule::string => Value::String(pair.into_inner().as_str().into()),
            Rule::boolean => Value::Boolean(pair.as_str().parse().unwrap()),
            _ => panic!("Unhandled AttributeValue pair {pair:?}"),
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
                            value = MarkupType::Value(val);
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

#[derive(Debug, Clone, Default)]
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    pub fn get(&self, name: &str) -> Option<Attribute> {
        for attr in &self.0 {
            if attr.name.as_str() == name {
                return Some(attr.clone());
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

impl FromIterator<Attribute> for Attributes {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        Self(c)
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
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
