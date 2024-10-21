use std::any::type_name;
use std::borrow::Borrow;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use arbutus::{NodeBuilder, NodeRef, TreeBuilder};
use attribute::AttributeParser;
use parking_lot::Mutex;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::{debug, debug_span, error};

use crate::attribute::{Attribute, Attributes};
use crate::data::provider::Provider;
use crate::data::url_provider::UrlProvider;
use crate::data::DataType;

#[cfg(not(target_arch = "wasm32"))]
use crate::data::FileProvider;
use crate::message::{Event, WidgetMessage};
use crate::node::{SnowcapNode, SnowcapNodeData};
use crate::{NodeId, Tree};

pub(crate) mod attribute;
pub(crate) mod color;
pub(crate) mod error;
pub(crate) mod gradient;
mod hash;

use error::ParseError;

#[derive(Parser)]
#[grammar = "snowcap.pest"]
pub struct SnowcapParser<'a, M> {
    _phantom: PhantomData<&'a M>,
}

type SnowNodeBuilder<'a, M> = NodeBuilder<
    'a,
    SnowcapNode<M>,
    ParseError,
    arbutus::IdGenerator,
    crate::Node<SnowcapNode<M>, crate::NodeId>,
    crate::NodeRef<M>,
>;

/// Snowcap parser
///
/// This parser uses Pest grammar to parse snowcap text into TreeNodes
impl<'a, M> SnowcapParser<'a, M>
where
    M: Clone + std::fmt::Debug + From<Event> + From<(NodeId, WidgetMessage)>,
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
    pub fn parse_file(filename: &Path) -> Result<Tree<M>, crate::Error> {
        tracing::info!("Parsing file {filename:?}");
        let data = std::fs::read_to_string(filename).expect("cannot read file");
        SnowcapParser::<M>::parse_memory(data.as_str())
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
    pub fn parse_memory(data: &str) -> Result<Tree<M>, crate::Error> {
        debug_span!("parser").in_scope(|| {
            let markup = SnowcapParser::<M>::parse(Rule::markup, data)?
                .next()
                .unwrap();

            let mut builder = TreeBuilder::<
                SnowcapNode<M>,
                ParseError,
                arbutus::IdGenerator,
                crate::Node<SnowcapNode<M>, crate::NodeId>,
                crate::NodeRef<M>,
            >::new();

            let root = SnowcapNode::<M>::new(SnowcapNodeData::Root);

            builder = builder.root(root, |root| {
                let _nodes = SnowcapParser::<M>::parse_pair(markup, root)?;
                Ok(())
            })?;

            debug!("Parsing complete");

            let tree = builder.done()?;

            Ok(tree.unwrap())
        })
    }

    fn parse_attributes(pair: Pair<Rule>) -> Result<Attributes, ParseError> {
        AttributeParser::parse_attributes(pair.as_str())
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
                        return Ok(Value::Dynamic {
                            data: Some(data),
                            provider: None,
                        });
                    }
                    "url" => {
                        let provider = UrlProvider::new(value.as_str())?;
                        return Ok(Value::Dynamic {
                            data: None,
                            provider: Some(Arc::new(Mutex::new(provider))),
                        });
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    "file" => {
                        let path = &PathBuf::from(value.clone());
                        let provider = FileProvider::new(path).unwrap();
                        return Ok(Value::Dynamic {
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

    fn parse_container<'b>(
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let inner = pair.into_inner();

        let mut id = None;
        let mut attrs: Option<Attributes> = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::id => {
                    let container_id = pair.into_inner().as_str();
                    tracing::info!("Element List ID {container_id}");
                    id = Some(container_id.to_string());
                }
                Rule::row | Rule::column | Rule::widget | Rule::stack => {
                    let node = SnowcapNode::new(SnowcapNodeData::Container)
                        .with_element_id(id)
                        .with_attrs(attrs);

                    builder.child(node, |container| {
                        Self::parse_pair(pair, container)?;
                        Ok(())
                    })?;

                    // Use a return here so we don't have to clone id and attrs
                    return Ok(());
                }
                Rule::attributes => {
                    attrs = Some(Self::parse_attributes(pair)?);
                    debug!("Container attributes {attrs:?}");
                }
                _ => return Err(ParseError::UnsupportedRule(format!("{:?}", pair.as_rule()))),
            };
        }
        Ok(())
    }

    fn parse_element_list<'b>(
        pairs: Pairs<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(ElementIdOption, Attributes), ParseError> {
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
                    attrs = Self::parse_attributes(pair)?;
                }
                _ => {
                    SnowcapParser::<M>::parse_pair(pair, builder)?;
                }
            }
        }

        Ok((id, attrs))
    }

    fn parse_row<'b>(
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Row);

        builder.child(node, |row| {
            debug!("Parsing column contents");
            let (id, attrs) = Self::parse_element_list(pair.into_inner(), row)?;
            row.node_mut()
                .with_data_mut(|mut data| {
                    data.element_id = id;
                    data.attrs = Some(attrs);
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    fn parse_column<'b>(
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Column);

        builder.child(node, |col| {
            debug!("Parsing column contents");
            let (id, attrs) = Self::parse_element_list(pair.into_inner(), col)?;
            col.node_mut()
                .with_data_mut(|mut data| {
                    data.element_id = id;
                    data.attrs = Some(attrs);
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    fn parse_stack<'b>(
        pair: Pair<Rule>,
        builder: &'b mut SnowNodeBuilder<'_, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Stack);

        builder.child(node, |stack| {
            debug!("Parsing column contents");
            let (id, attrs) = Self::parse_element_list(pair.into_inner(), stack)?;
            stack
                .node_mut()
                .with_data_mut(|mut data| {
                    data.element_id = id;
                    data.attrs = Some(attrs);
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    fn parse_widget<'b>(
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let mut inner = pair.into_inner();
        let label = inner.next().unwrap().as_str().to_string();

        let node = SnowcapNode::new(SnowcapNodeData::Widget(label));

        builder.child(node, |widget| {
            for pair in inner {
                match pair.as_rule() {
                    Rule::id => {
                        let widget_id = pair.into_inner().as_str();
                        widget
                            .node_mut()
                            .with_data_mut(|mut data| {
                                data.element_id = Some(widget_id.to_string());
                                Ok::<(), ()>(())
                            })
                            .ok();
                    }
                    Rule::attributes => {
                        let attrs = Self::parse_attributes(pair)?;
                        widget
                            .node_mut()
                            .with_data_mut(|mut data| {
                                data.attrs = Some(attrs);
                                Ok::<(), ()>(())
                            })
                            .ok();
                    }
                    Rule::element_value => {
                        let value = Self::parse_value(pair.into_inner().next().unwrap())?;
                        let node = SnowcapNode::new(SnowcapNodeData::Value(value));

                        widget.child(node, |_| Ok(()))?;
                    }
                    Rule::array => {
                        let value = Self::parse_array(pair)?;
                        let node = SnowcapNode::new(SnowcapNodeData::Value(value));

                        widget.child(node, |_| Ok(()))?;
                    }
                    Rule::widget => {
                        Self::parse_pair(pair, widget)?;
                    }
                    Rule::container => {
                        Self::parse_container(pair, widget)?;
                    }
                    _ => {
                        error!("Unhandled element rule {:?}", pair.as_rule())
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    pub(crate) fn parse_pair<'b>(
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        match pair.as_rule() {
            Rule::container => Self::parse_container(pair, builder),
            Rule::row => Self::parse_row(pair, builder),
            Rule::column => Self::parse_column(pair, builder),
            Rule::stack => Self::parse_stack(pair, builder),
            Rule::widget => Self::parse_widget(pair, builder),
            //Rule::element_value => Self::parse_pair(pair.into_inner().last().unwrap()),
            Rule::element_value => Self::parse_pair(pair.into_inner().last().unwrap(), builder),
            _ => panic!("Unhandled {pair:?}"),
        }?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Dynamic {
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
            Value::Dynamic { provider, .. } => write!(f, "[Dynamic provider={provider:?}]"),
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

#[cfg(test)]
mod tests {}
