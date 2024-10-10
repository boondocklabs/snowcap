use std::any::type_name;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::{debug, debug_span, error, warn};

use crate::attribute::{Attribute, Attributes};
use crate::data::provider::Provider;
use crate::data::url_provider::UrlProvider;
use crate::data::DataType;

#[cfg(not(target_arch = "wasm32"))]
use crate::data::FileProvider;
use crate::message::{Event, WidgetMessage};
use crate::tree::node::TreeNode;

pub(crate) mod color;
pub(crate) mod error;
pub(crate) mod gradient;

use error::ParseError;

#[derive(Parser)]
#[grammar = "snowcap.pest"]
pub struct SnowcapParser<'a, M> {
    _phantom: PhantomData<&'a M>,
}

/// Snowcap parser
///
/// This parser uses Pest grammar to parse snowcap text into TreeNodes
impl<'a, M> SnowcapParser<'a, M>
where
    M: Clone + std::fmt::Debug + From<Event> + From<WidgetMessage>,
{
    /// Parse a file into a `TreeNode`.
    ///
    /// # Arguments
    ///
    /// * `filename`: The path to the file containing Snowcap text.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `TreeNode`, or an error if parsing fails.
    pub fn parse_file(filename: &Path) -> Result<TreeNode<'a, M>, crate::Error> {
        tracing::info!("Parsing file {filename:?}");
        let data = std::fs::read_to_string(filename).expect("cannot read file");
        SnowcapParser::parse_memory(data.as_str())
    }

    /// Parse Snowcap string into a `TreeNode`.
    ///
    /// # Arguments
    ///
    /// * `data`: The Snowcap text to be parsed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the root `TreeNode`, or an error if parsing fails.
    pub fn parse_memory(data: &str) -> Result<TreeNode<'a, M>, crate::Error> {
        debug_span!("parser").in_scope(|| {
            let markup = SnowcapParser::<M>::parse(Rule::markup, data)?
                .next()
                .unwrap();
            let nodes = SnowcapParser::parse_pair(markup)?;

            debug!("Pairs parsed. Building tree.");

            let root = TreeNode::<'a, M>::new(nodes);

            debug!("Parsing complete");

            Ok(root)
        })
    }

    fn parse_attributes(pairs: Pairs<Rule>) -> Attributes {
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
                Attribute::new(name, value)
            })
            .collect()
    }

    fn parse_array(pair: Pair<Rule>) -> Result<Value, ParseError> {
        let values: Result<Vec<Value>, ParseError> =
            pair.into_inner().map(|i| Self::parse_value(i)).collect();

        Ok(Value::Array(values?))
    }

    fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
        debug!("value {:?} {}", pair.as_rule(), pair.as_str());
        match pair.as_rule() {
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

                match name.as_str() {
                    "qr" => {
                        let data = iced::widget::qr_code::Data::new(value)?;
                        let data = Arc::new(DataType::QrCode(Arc::new(data)));
                        return Ok(Value::Data {
                            data: Some(data),
                            provider: None,
                        });
                    }
                    "url" => {
                        let provider = UrlProvider::new(value.as_str())?;
                        return Ok(Value::Data {
                            data: None,
                            provider: Some(Arc::new(Mutex::new(provider))),
                        });
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    "file" => {
                        let path = &PathBuf::from(value.clone());
                        let provider = FileProvider::new(path).unwrap();
                        return Ok(Value::Data {
                            data: None,
                            provider: Some(Arc::new(Mutex::new(provider))),
                        });
                    }
                    _ => return Err(ParseError::Unhandled("Missing data or provider".into())),
                };
            }
            _ => Err(ParseError::Unhandled(format!(
                "AttributeValue {:?}",
                pair.as_rule()
            ))),
        }
    }

    fn parse_container(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        let inner = pair.into_inner();

        let mut content: MarkupTreeNode<'a, M> = MarkupTreeNode::None;
        let mut attr = Attributes::default();
        let mut id = None;

        for pair in inner {
            debug!(
                "{} {:#?} {}",
                type_name::<Self>(),
                pair.as_rule(),
                pair.as_str()
            );
            match pair.as_rule() {
                Rule::id => {
                    let container_id = pair.into_inner().as_str();
                    tracing::info!("Element List ID {container_id}");
                    id = Some(container_id.to_string());
                }
                Rule::row | Rule::column | Rule::widget | Rule::stack => {
                    content = Self::parse_pair(pair)?;
                }
                Rule::attributes => {
                    attr = Self::parse_attributes(pair.into_inner());
                    debug!("Container attributes {attr:?}");
                }
                _ => return Err(ParseError::UnsupportedRule(format!("{:?}", pair.as_rule()))),
            };
        }

        Ok(MarkupTreeNode::Container {
            element_id: id,
            content: TreeNode::new(content),
            attrs: attr,
        })
    }

    fn parse_element_list(
        pairs: Pairs<Rule>,
    ) -> Result<(ElementIdOption, Attributes, Vec<TreeNode<'a, M>>), ParseError> {
        let mut contents = Vec::new();
        let mut attrs = Attributes::default();
        let mut id: Option<String> = None;

        for pair in pairs {
            match &pair.as_rule() {
                Rule::id => {
                    let list_id = pair.into_inner().as_str();
                    tracing::info!("Element List ID {list_id}");
                    id = Some(list_id.to_string());
                }
                Rule::attributes => {
                    attrs = Self::parse_attributes(pair.into_inner());
                }
                _ => contents.push(TreeNode::new(SnowcapParser::parse_pair(pair)?)),
            }
        }

        Ok((id, attrs, contents))
    }

    fn parse_row(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTreeNode::Row {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_column(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTreeNode::Column {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_stack(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTreeNode::Stack {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_widget(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        let mut inner = pair.into_inner();
        let label = inner.next().unwrap().as_str().to_string();

        let mut attr = Attributes::default();
        let mut value = MarkupTreeNode::None;
        let mut id: ElementIdOption = None;

        for pair in inner {
            debug!("widget {:?} {}", pair.as_rule(), pair.as_str());
            match pair.as_rule() {
                Rule::id => {
                    let widget_id = pair.into_inner().as_str();
                    tracing::info!("Widget ID {widget_id}");
                    id = Some(widget_id.to_string());
                }
                Rule::attributes => {
                    attr = Self::parse_attributes(pair.into_inner());
                }
                Rule::element_value => {
                    let val = Self::parse_value(pair.into_inner().next().unwrap());
                    value = MarkupTreeNode::Value(Arc::new(RefCell::new(val.unwrap())));
                }
                Rule::widget => {
                    value = Self::parse_pair(pair)?;
                }
                Rule::container => {
                    value = Self::parse_container(pair).unwrap();
                }
                Rule::array => {
                    let val = Self::parse_array(pair);
                    value = MarkupTreeNode::Value(Arc::new(RefCell::new(val.unwrap())));
                }
                _ => {
                    error!("Unhandled element rule {:?}", pair.as_rule())
                }
            }
        }

        Ok(MarkupTreeNode::Widget {
            element_id: id,
            name: label,
            attrs: attr,
            content: TreeNode::new(value),
        })
    }

    pub(crate) fn parse_pair(pair: Pair<Rule>) -> Result<MarkupTreeNode<'a, M>, ParseError> {
        debug!("{:?} {:?}", pair.as_rule(), pair.as_str());
        match pair.as_rule() {
            Rule::container => Self::parse_container(pair),
            Rule::row => Self::parse_row(pair),
            Rule::column => Self::parse_column(pair),
            Rule::stack => Self::parse_stack(pair),
            Rule::widget => Self::parse_widget(pair),
            Rule::element_value => Self::parse_pair(pair.into_inner().last().unwrap()),
            Rule::label => Ok(MarkupTreeNode::Label(
                pair.into_inner().next().unwrap().as_str().into(),
            )),
            _ => panic!("Unhandled {pair:?}"),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Data {
        data: Option<Arc<DataType>>,
        provider: Option<Arc<Mutex<dyn Provider>>>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => f.write_str(s),
            Value::Number(num) => f.write_fmt(format_args!("{}", num)),
            Value::Boolean(b) => f.write_fmt(format_args!("{}", b)),
            Value::Array(_vec) => f.write_str("[vec...]"),
            Value::Data { .. } => f.write_str("[Data]"),
            /*
            Value::DataSource {
                name,
                value: _,
                provider: _,
            } => f.write_fmt(format_args!("Data source [{}]", name)),
            */
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare `String` variants
            (Value::String(a), Value::String(b)) => a == b,
            // Compare `Number` variants
            (Value::Number(a), Value::Number(b)) => a == b,
            // Compare `Boolean` variants
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            // Compare `Null` variants
            // Compare `Array` variants (recursively compares each element)
            (Value::Array(a), Value::Array(b)) => a == b,

            // Return false for types we can't compare
            _ => false,
        }
    }
}

impl Borrow<String> for Value {
    fn borrow(&self) -> &String {
        match self {
            Value::String(s) => &s,
            _ => panic!("Cannot borrow string for non-string typed value"),
        }
    }
}

pub type ElementId = String;
type ElementIdOption = Option<ElementId>;

/// Abstract Syntax Tree (AST) representation of the parsed grammar
#[derive(Debug, Clone)]
pub enum MarkupTreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    None,
    Container {
        element_id: ElementIdOption,
        attrs: Attributes,
        content: TreeNode<'a, M>,
    },
    Widget {
        element_id: ElementIdOption,
        name: String,
        attrs: Attributes,
        content: TreeNode<'a, M>,
    },
    Row {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    },
    Column {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    },
    Stack {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<'a, M>>>,
    },
    Label(String),
    Value(Arc<RefCell<Value>>),
    Phantom(PhantomData<M>),
}

impl<'a, M> MarkupTreeNode<'a, M>
where
    M: std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn get_element_id(&self) -> &ElementIdOption {
        match self {
            MarkupTreeNode::Container { element_id, .. } => element_id,
            MarkupTreeNode::Widget { element_id, .. } => element_id,
            MarkupTreeNode::Row { element_id, .. } => element_id,
            MarkupTreeNode::Column { element_id, .. } => element_id,
            MarkupTreeNode::Stack { element_id, .. } => element_id,
            _ => {
                warn!("Get element ID node on node type without element_id");
                &None
            }
        }
    }
    pub fn set_element_id(&mut self, new_element_id: ElementIdOption) {
        match self {
            MarkupTreeNode::Container { element_id, .. } => *element_id = new_element_id,
            MarkupTreeNode::Widget { element_id, .. } => *element_id = new_element_id,
            MarkupTreeNode::Row { element_id, .. } => *element_id = new_element_id,
            MarkupTreeNode::Column { element_id, .. } => *element_id = new_element_id,
            MarkupTreeNode::Stack { element_id, .. } => *element_id = new_element_id,
            _ => warn!("Set element ID node on node type without element_id"),
        }
    }

    pub fn get_content(&'a self) -> Option<TreeNode<M>>
    where
        M: Clone + std::fmt::Debug,
    {
        match self {
            MarkupTreeNode::Widget { content, .. } => Some(content.clone()),
            _ => {
                warn!("No content for widget");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {}
