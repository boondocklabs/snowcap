use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arbutus::{NodeBuilder, NodeRef, TreeBuilder};
use attribute::AttributeParser;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::{debug, debug_span, error};
use value::ValueKind;

use crate::attribute::Attributes;
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
pub(crate) mod value;

pub use value::Value;

use error::{ParseError, ParseErrorContext};

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

#[derive(Parser)]
#[grammar = "snowcap.pest"]
pub struct SnowcapParser<M> {
    context: ParserContext,
    _phantom: PhantomData<M>,
}

impl<M> Default for SnowcapParser<M> {
    fn default() -> Self {
        Self {
            context: ParserContext::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> SnowcapParser<M>
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
        SnowcapParser::<M>::parse_memory(data.as_str()).map_err(|e| crate::Error::Parse(e))
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
    pub fn parse_memory(data: &str) -> Result<Tree<M>, ParseErrorContext> {
        debug_span!("parser").in_scope(|| {
            let markup = SnowcapParser::<M>::parse(Rule::markup, data)
                .map_err(|e| {
                    let mut context = ParserContext::default();
                    match e.line_col {
                        pest::error::LineColLocation::Pos(pos) => context.location = pos,
                        pest::error::LineColLocation::Span(_, _) => todo!(),
                    }
                    context.input = data.into();
                    ParseErrorContext::new(context, ParseError::from(e))
                })?
                .next()
                .unwrap();

            // Initialize parser context
            let mut parser = Self::default().context((&markup).into());

            let mut builder = TreeBuilder::<
                SnowcapNode<M>,
                ParseError,
                arbutus::IdGenerator,
                crate::Node<SnowcapNode<M>, crate::NodeId>,
                crate::NodeRef<M>,
            >::new();

            let root = SnowcapNode::<M>::new(SnowcapNodeData::Root);

            builder = builder
                .root(root, |root| parser.parse_pair(markup, root))
                .map_err(|e| ParseErrorContext::new(parser.context.clone(), e))?;

            debug!("Parsing complete");

            let tree = builder
                .done()
                .map_err(|e| ParseErrorContext::new(parser.context.clone(), e))?;

            Ok(tree.unwrap())
        })
    }

    pub fn context(mut self, context: ParserContext) -> Self {
        self.context = context;
        self
    }

    fn parse_attributes(pair: Pair<Rule>) -> Result<Attributes, ParseError> {
        AttributeParser::parse_attributes(pair.as_str())
    }

    fn parse_array(pair: Pair<Rule>) -> Result<Value, ParseError> {
        let context = ParserContext::from(&pair);

        let values: Result<Vec<Value>, ParseError> = pair
            .into_inner()
            .map(|i| Self::parse_value(i.clone()).map(|v| v.with_context(ParserContext::from(&i))))
            .collect();

        Ok(Value::new_array(values?).with_context(context))
    }

    fn parse_value(pair: Pair<Rule>) -> Result<Value, ParseError> {
        let context = ParserContext::from(&pair);

        let res = {
            debug!("value {:?} {}", pair.as_rule(), pair.as_str());
            match pair.as_rule() {
                Rule::number => {
                    Ok(Value::new_float(pair.as_str().parse().unwrap()).with_context(context))
                }
                Rule::string => {
                    Ok(Value::new_string(pair.into_inner().as_str().into()).with_context(context))
                }
                Rule::boolean => {
                    Ok(Value::new_bool(pair.as_str().parse().unwrap()).with_context(context))
                }
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
                            let data = DataType::QrCode(Arc::new(data));
                            Ok(Value::new_data(data).with_context(context))
                        }
                        "url" => {
                            let provider = UrlProvider::new(value.as_str())?;
                            Ok(Value::new_provider(provider).with_context(context))
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        "file" => {
                            let path = &PathBuf::from(value.clone());
                            let provider = FileProvider::new(path)?;
                            Ok(Value::new_provider(provider).with_context(context))
                        }
                        _ => return Err(ParseError::Unhandled("Missing data or provider".into())),
                    }
                }
                _ => Err(ParseError::Unhandled(format!(
                    "AttributeValue {:?}",
                    pair.as_rule()
                ))),
            }
        };

        res.map_err(|e| e)
    }

    fn parse_container<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let inner = pair.into_inner();

        let mut id = None;
        let mut attrs: Option<Attributes> = None;

        for pair in inner {
            self.context = ParserContext::from(&pair);
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
                        self.parse_pair(pair, container)?;
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
        &mut self,
        pairs: Pairs<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(ElementIdOption, Attributes), ParseError> {
        let mut attrs = Attributes::default();
        let mut id: Option<String> = None;

        for pair in pairs {
            self.context = ParserContext::from(&pair);
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
                    self.parse_pair(pair, builder)?;
                }
            }
        }

        Ok((id, attrs))
    }

    fn parse_row<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Row);

        builder.child(node, |row| {
            debug!("Parsing column contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), row)?;
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
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Column);

        builder.child(node, |col| {
            debug!("Parsing column contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), col)?;
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
        &mut self,
        pair: Pair<Rule>,
        builder: &'b mut SnowNodeBuilder<'_, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(SnowcapNodeData::Stack);

        builder.child(node, |stack| {
            debug!("Parsing column contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), stack)?;
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
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let mut inner = pair.into_inner();
        let label = inner.next().unwrap().as_str().to_string();

        let node = SnowcapNode::new(SnowcapNodeData::Widget(label));

        builder.child(node, |widget| {
            for pair in inner {
                self.context = ParserContext::from(&pair);
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
                        self.parse_pair(pair, widget)?;
                    }
                    Rule::container => {
                        self.parse_container(pair, widget)?;
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
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        self.context = (&pair).into();

        match pair.as_rule() {
            Rule::container => self.parse_container(pair, builder),
            Rule::row => self.parse_row(pair, builder),
            Rule::column => self.parse_column(pair, builder),
            Rule::stack => self.parse_stack(pair, builder),
            Rule::widget => self.parse_widget(pair, builder),
            Rule::element_value => self.parse_pair(pair.into_inner().last().unwrap(), builder),
            _ => panic!("Unhandled {pair:?}"),
        }?;

        Ok(())
    }
}

/// Context information stored in tree nodes by the parser
/// to provide location information from the parsed markup
#[derive(Clone, Debug, Default)]
pub struct ParserContext {
    input: String,
    location: (usize, usize),
}

impl From<&Pair<'_, Rule>> for ParserContext {
    fn from(pair: &Pair<'_, Rule>) -> Self {
        ParserContext {
            input: pair.get_input().into(),
            location: pair.line_col(),
        }
    }
}

pub type ElementId = String;
type ElementIdOption = Option<ElementId>;

#[cfg(test)]
mod tests {}
