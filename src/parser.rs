use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use iced::Element;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::{debug, error};

use crate::attribute::{Attribute, Attributes};
use crate::data::provider::Provider;
use crate::data::url_provider::UrlProvider;
use crate::data::DataType;
use crate::Message;

#[cfg(not(target_arch = "wasm32"))]
use crate::data::FileProvider;

pub(crate) mod color;
pub(crate) mod error;
pub(crate) mod gradient;

use error::ParseError;

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Parser)]
#[grammar = "snowcap.pest"]
pub struct SnowcapParser<AppMessage> {
    _phantom: PhantomData<AppMessage>,
}

impl<AppMessage> SnowcapParser<AppMessage> {
    pub fn parse_file(filename: &Path) -> Result<TreeNode<AppMessage>, crate::Error> {
        tracing::info!("Parsing file {filename:?}");
        let data = std::fs::read_to_string(filename).expect("cannot read file");
        SnowcapParser::parse_memory(data.as_str())
    }

    pub fn parse_memory(data: &str) -> Result<TreeNode<AppMessage>, crate::Error> {
        let markup = SnowcapParser::<AppMessage>::parse(Rule::markup, data)?
            .next()
            .unwrap();
        Ok(TreeNode::new(SnowcapParser::parse_pair(markup)?))
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

    fn parse_container(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        debug!("[Parsing Container]");

        let inner = pair.into_inner();

        let mut content: MarkupTree<AppMessage> = MarkupTree::None;
        let mut attr = Attributes::default();

        for pair in inner {
            match pair.as_rule() {
                Rule::row | Rule::column | Rule::widget | Rule::stack => {
                    content = Self::parse_pair(pair)?;
                }
                Rule::attributes => {
                    attr = Self::parse_attributes(pair.into_inner());
                    debug!("{attr:?}");
                }
                _ => return Err(ParseError::UnsupportedRule(format!("{:?}", pair.as_rule()))),
            };
        }

        Ok(MarkupTree::Container {
            content: TreeNode::new(content),
            attrs: attr,
        })
    }

    fn parse_element_list(
        pairs: Pairs<Rule>,
    ) -> Result<(ElementIdOption, Attributes, Vec<TreeNode<AppMessage>>), ParseError> {
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

    fn parse_row(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTree::Row {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_column(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTree::Column {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_stack(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        let (id, attrs, contents) = Self::parse_element_list(pair.into_inner())?;

        Ok(MarkupTree::Stack {
            element_id: id,
            attrs,
            contents: Arc::new(contents),
        })
    }

    fn parse_widget(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        let mut inner = pair.into_inner();
        let label = inner.next().unwrap().as_str().to_string();

        let mut attr = Attributes::default();
        let mut value = MarkupTree::None;
        let mut id: ElementIdOption = None;

        for pair in inner {
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
                    value = MarkupTree::Value(Arc::new(RefCell::new(val.unwrap())));
                }
                Rule::widget => {
                    value = Self::parse_pair(pair)?;
                }
                Rule::container => {
                    value = Self::parse_container(pair).unwrap();
                }
                Rule::array => {
                    let val = Self::parse_array(pair);
                    value = MarkupTree::Value(Arc::new(RefCell::new(val.unwrap())));
                }
                _ => {
                    error!("Unhandled element rule {:?}", pair.as_rule())
                }
            }
        }

        Ok(MarkupTree::Widget {
            element_id: id,
            name: label,
            attrs: attr,
            content: TreeNode::new(value),
        })
    }

    pub(crate) fn parse_pair(pair: Pair<Rule>) -> Result<MarkupTree<AppMessage>, ParseError> {
        match pair.as_rule() {
            Rule::container => Self::parse_container(pair),
            Rule::row => Self::parse_row(pair),
            Rule::column => Self::parse_column(pair),
            Rule::stack => Self::parse_stack(pair),
            Rule::widget => Self::parse_widget(pair),
            Rule::element_value => Self::parse_pair(pair.into_inner().last().unwrap()),
            Rule::label => Ok(MarkupTree::Label(
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

pub type NodeId = u64;

/// Type alias for inner tree nodes wrapped in Arc
pub type TreeNodeInner<M> = Arc<MarkupTree<M>>;

pub struct TreeNode<AppMessage> {
    id: NodeId,
    inner: TreeNodeInner<AppMessage>,
    pub element: Arc<RwLock<Option<Arc<Element<'static, Message<AppMessage>>>>>>,
    pub data: Arc<RwLock<Option<DataType>>>,
}

impl<AppMessage> std::fmt::Debug for TreeNode<AppMessage> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TreeNode ID={}", self.id))
    }
}

impl<M> Clone for TreeNode<M> {
    fn clone(&self) -> Self {
        TreeNode {
            id: self.id,
            inner: self.inner.clone(),
            element: Arc::new(RwLock::new(None)),
            data: Arc::new(RwLock::new(None)),
        }
    }
}

impl<AppMessage> TreeNode<AppMessage> {
    pub fn new(inner: MarkupTree<AppMessage>) -> Self {
        Self {
            id: NEXT_NODE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            inner: Arc::new(inner),
            element: Arc::new(RwLock::new(None)),
            data: Arc::new(RwLock::new(None)),
        }
    }

    pub fn element_read(
        &self,
    ) -> RwLockReadGuard<Option<Arc<Element<'static, Message<AppMessage>>>>> {
        self.element.read()
    }

    pub fn inner<'a>(&'a self) -> &MarkupTree<AppMessage> {
        &self.inner
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn element_id(&self) -> &Option<ElementId> {
        match self.inner() {
            MarkupTree::Container {
                attrs: _,
                content: _,
            } => &None,
            MarkupTree::Widget { element_id: id, .. } => id,
            MarkupTree::Row { element_id: id, .. } => id,
            MarkupTree::Column { element_id: id, .. } => id,
            MarkupTree::Stack { element_id: id, .. } => id,
            _ => &None,
        }
    }
}

pub type ElementId = String;
type ElementIdOption = Option<ElementId>;

/// Abstract Syntax Tree (AST) representation of the parsed grammar
#[derive(Debug)]
pub enum MarkupTree<AppMessage> {
    None,
    Container {
        attrs: Attributes,
        content: TreeNode<AppMessage>,
    },
    Widget {
        element_id: ElementIdOption,
        name: String,
        attrs: Attributes,
        content: TreeNode<AppMessage>,
    },
    Row {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<AppMessage>>>,
    },
    Column {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<AppMessage>>>,
    },
    Stack {
        element_id: ElementIdOption,
        attrs: Attributes,
        contents: Arc<Vec<TreeNode<AppMessage>>>,
    },
    Label(String),
    Value(Arc<RefCell<Value>>),
    Phantom(PhantomData<AppMessage>),
}

impl<AppMessage> TreeNode<AppMessage> {
    // Define an `into_iter` method that returns an iterator over Arc-wrapped nodes
    pub fn into_iter(self) -> MarkupTreeIter<AppMessage> {
        MarkupTreeIter {
            stack: VecDeque::from([self.into()]), // Start with the root node in Arc
        }
    }
}

impl<AppMessage> IntoIterator for TreeNode<AppMessage> {
    type Item = TreeNode<AppMessage>;
    type IntoIter = MarkupTreeIter<AppMessage>;

    fn into_iter(self) -> Self::IntoIter {
        MarkupTreeIter {
            stack: VecDeque::from([self]),
        }
    }
}

pub struct MarkupTreeIter<AppMessage> {
    stack: VecDeque<TreeNode<AppMessage>>,
}

impl<AppMessage> Iterator for MarkupTreeIter<AppMessage> {
    type Item = TreeNode<AppMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop_front();

        if let Some(tree) = current.clone() {
            match &*tree.inner {
                MarkupTree::Container { content, .. } | MarkupTree::Widget { content, .. } => {
                    self.stack.push_front(content.clone());
                }
                MarkupTree::Row { contents, .. }
                | MarkupTree::Column { contents, .. }
                | MarkupTree::Stack { contents, .. } => {
                    for content in contents.iter().rev() {
                        self.stack.push_front(content.clone());
                    }
                }
                _ => {}
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {}
