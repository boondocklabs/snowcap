//! The parsers process Snowcap grammar and produces an [`arbutus::Tree`]

use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;

use arbutus::{NodeBuilder, TreeBuilder, TreeNodeRef};
use attribute::AttributeParser;
use module::ModuleParser;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use tracing::{debug, debug_span};
use value::{ValueData, ValueParser};

use crate::attribute::Attributes;

use crate::message::{Event, WidgetMessage};
use crate::node::{Content, SnowcapNode};
use crate::{NodeId, Tree};

pub(crate) mod attribute;
pub(crate) mod color;
pub(crate) mod error;
pub(crate) mod gradient;
mod hash;
pub(crate) mod module;
pub(crate) mod value;

pub use value::Value;

#[cfg(test)]
mod test;

use error::{ParseError, ParseErrorContext};

type SnowNodeBuilder<'a, M> = NodeBuilder<
    'a,
    SnowcapNode<M>,
    ParseError,
    arbutus::IdGenerator,
    crate::Node<SnowcapNode<M>, crate::NodeId>,
    crate::NodeRef<M>,
>;

/// Parses a top level Snowcap grammar and produces an [`arbutus::Tree`] of [`crate::node::SnowcapNode`] nodes.
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
    /// Parse a Snowcap file into an [`arbutus::Tree`].
    ///
    /// # Arguments
    ///
    /// * `filename`: The path to the file containing Snowcap text.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed [`arbutus::Tree`], or a [`crate::Error`] if parsing fails.
    pub fn parse_file(filename: &Path) -> Result<Tree<M>, crate::Error> {
        tracing::info!("Parsing file {filename:?}");
        let data = std::fs::read_to_string(filename).expect("cannot read file");
        SnowcapParser::<M>::parse_memory(data.as_str()).map_err(|e| crate::Error::Parse(e))
    }

    /// Parse a Snowcap string from memory into an [`arbutus::Tree`].
    ///
    /// # Arguments
    ///
    /// * `data`: The Snowcap text to be parsed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed [`arbutus::Tree`], or a [`crate::Error`] if parsing fails.
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

            let root = SnowcapNode::<M>::new(Content::Root);

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

    /// Parse [`Attributes`] from the pairs
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed [`Attributes`], or [`ParseError`] on failure
    fn parse_attributes(pair: Pair<Rule>) -> Result<Attributes, ParseError> {
        AttributeParser::parse_attributes(pair.as_str())
    }

    /// Parse [`Value`] from the pairs
    fn parse_value(&self, pair: Pair<Rule>) -> Result<Value, ParseError> {
        let context = ParserContext::from(&pair);
        debug!("value {:?} {}", pair.as_rule(), pair.as_str());
        ValueParser::parse_str(pair.as_str(), &context)
    }

    /// Parse a Container widget.
    ///
    /// Parses the ID and [`Attributes`] for this container,
    /// and one of a `row`, `column`, `stack`, or `widget`
    /// as the contents of the container, and adds it
    /// as a child of the parent using the supplied NodeBuilder.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
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
                    let node = SnowcapNode::new(Content::Container)
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
                Rule::module => {
                    self.parse_module(pair, builder)?;
                }
                _ => {
                    return Err(ParseError::UnsupportedRule(format!(
                        "{}: {} {:?}",
                        file!(),
                        line!(),
                        pair.as_rule()
                    )))
                }
            };
        }
        Ok(())
    }

    /// Parse an element list (used for row, column, and stack) as these
    /// types can accept multiple elements as their contents.
    fn parse_element_list<'b>(
        &mut self,
        pairs: Pairs<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(ElementIdOption, Option<Attributes>), ParseError> {
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

        if attrs.len() > 0 {
            Ok((id, Some(attrs)))
        } else {
            Ok((id, None))
        }
    }

    /// Parse a Row widget.
    ///
    /// Parses the ID and [`Attributes`] for this row, and an element list as the contents.
    /// The row is then added as a child of the parent using the supplied NodeBuilder.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
    fn parse_row<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(Content::Row);

        builder.child(node, |row| {
            debug!("Parsing row contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), row)?;
            row.node_mut()
                .with_data_mut(|data| {
                    data.element_id = id;
                    data.attrs = attrs;
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    /// Parse a Column widget.
    ///
    /// Parses the ID and [`Attributes`] for this column, and an element list as the contents.
    /// The column is then added as a child of the parent using the supplied NodeBuilder.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
    fn parse_column<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(Content::Column);

        builder.child(node, |col| {
            debug!("Parsing column contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), col)?;
            col.node_mut()
                .with_data_mut(|data| {
                    data.element_id = id;
                    data.attrs = attrs;
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    /// Parse a Stack widget.
    ///
    /// Parses the ID and [`Attributes`] for this stack, and an element list as the contents.
    /// The stack is then added as a child of the parent using the supplied NodeBuilder.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
    fn parse_stack<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &'b mut SnowNodeBuilder<'_, M>,
    ) -> Result<(), ParseError> {
        let node = SnowcapNode::new(Content::Stack);

        builder.child(node, |stack| {
            debug!("Parsing column contents");
            let (id, attrs) = self.parse_element_list(pair.into_inner(), stack)?;
            stack
                .node_mut()
                .with_data_mut(|data| {
                    data.element_id = id;
                    data.attrs = attrs;
                    Ok::<(), ()>(())
                })
                .ok();
            Ok(())
        })
    }

    /// Parse a generic widget.
    ///
    /// Parses the ID and [`Attributes`] for this widget, and recursively parses its content.
    /// The widget is then added as a child of the parent using the supplied NodeBuilder.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
    fn parse_widget<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        let mut inner = pair.into_inner();
        let label = inner.next().unwrap().as_str().to_string();

        let node = SnowcapNode::new(Content::Widget(label));

        builder.child(node, |widget| {
            for pair in inner {
                self.context = ParserContext::from(&pair);
                match pair.as_rule() {
                    Rule::id => {
                        let widget_id = pair.into_inner().as_str();
                        widget
                            .node_mut()
                            .with_data_mut(|data| {
                                data.element_id = Some(widget_id.to_string());
                                Ok::<(), ()>(())
                            })
                            .ok();
                    }
                    Rule::attributes => {
                        let attrs = Self::parse_attributes(pair)?;
                        widget
                            .node_mut()
                            .with_data_mut(|data| {
                                data.attrs = Some(attrs);
                                Ok::<(), ()>(())
                            })
                            .ok();
                    }
                    Rule::value => {
                        let value = self.parse_value(pair.into_inner().next().unwrap())?;
                        let node = SnowcapNode::new(Content::Value(value));

                        widget.child(node, |_| Ok(()))?;
                    }
                    Rule::widget => {
                        self.parse_pair(pair, widget)?;
                    }
                    Rule::container => {
                        self.parse_container(pair, widget)?;
                    }
                    Rule::module => {
                        self.parse_module(pair, widget)?;
                    }
                    _ => {
                        return Err(ParseError::UnsupportedRule(format!(
                            "{}: {} {:?}",
                            file!(),
                            line!(),
                            pair.as_rule()
                        )))
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    /// Parse a [`crate::parser::module::Module`], using a [`ModuleParser`].
    ///
    /// A new [`SnowcapNode`] with [`Content::Module`] containing the parsed module description
    /// is added as a child of the parent node.
    ///
    /// The supplied NodeBuilder provides the context of the parent node.
    ///
    fn parse_module<'b>(
        &mut self,
        pair: Pair<Rule>,
        builder: &mut SnowNodeBuilder<'b, M>,
    ) -> Result<(), ParseError> {
        // Parse the module
        let module = ModuleParser::parse_str(pair.as_str(), self.context.clone())?;

        // Add the module to the tree
        let node = SnowcapNode::new(Content::Module(module));
        builder.child(node, |_| Ok(()))?;

        Ok(())
    }

    /// Handle a [`Pair`], matching on the [`Rule`] from the PEG to dispatch it
    /// to one of the specific [`Pair`] parsing functions above.
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
            Rule::module => self.parse_module(pair, builder),
            Rule::element_value => self.parse_pair(pair.into_inner().last().unwrap(), builder),
            _ => {
                return Err(ParseError::UnsupportedRule(format!(
                    "{}: {} {:?}",
                    file!(),
                    line!(),
                    pair.as_rule()
                )))
            }
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
