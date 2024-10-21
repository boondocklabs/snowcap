use std::collections::HashMap;

use arbutus::Node as _;
use arbutus::NodeRef as _;
use tracing::debug;
use tracing::debug_span;
use tracing::info;

use crate::conversion::stack::SnowcapStack;
use crate::tree_util::ChildData;
use crate::{
    attribute::Attributes, message::WidgetMessage, node::SnowcapNodeData, ConversionError,
    DynamicWidget, NodeId, NodeRef,
};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, widget::SnowcapWidget,
};

impl<'a, M> DynamicWidget<'a, M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    fn content_single(
        _node_id: NodeId,
        children: Option<Vec<ChildData<'static, M>>>,
    ) -> Result<Option<DynamicWidget<'static, M>>, ConversionError> {
        if let Some(mut children) = children {
            if let Some(child) = children.pop() {
                match child {
                    ChildData::Widget(dynamic_widget) => Ok(Some(dynamic_widget)),
                    ChildData::Value(value) => match value {
                        crate::Value::String(_) => todo!(),
                        crate::Value::Number(_) => todo!(),
                        crate::Value::Boolean(_) => todo!(),
                        crate::Value::Array(_vec) => todo!(),
                        crate::Value::Dynamic { data: _, provider } => {
                            info!("RENDER DATA FOR {provider:?}");
                            Ok(Some(SnowcapWidget::loading()))
                        }
                    },
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn builder(
        node: NodeRef<M>,
        mut children: Option<HashMap<NodeId, ChildData<'static, M>>>,
    ) -> Result<DynamicWidget<'a, M>, ConversionError> {
        debug_span!("DynamicWidget").in_scope(|| {
            debug!("Building node_id={:?}", node.node().id());
            let widget = node.with_data(|data| {
                let node = node.node();
                let node_id = node.id();
                let attrs = data.attrs.clone();

                // Collect the contents in the order specified in the node
                let contents = children.as_mut().map(|children| {
                    let contents: Option<Vec<ChildData<M>>> = node.children().map(|child| {
                        child
                            .iter()
                            .map(|f| children.remove(f.node().id()).unwrap())
                            .collect()
                    });
                    contents.unwrap()
                });

                let widget = match &data.data {
                    SnowcapNodeData::None => todo!(),
                    SnowcapNodeData::Root => todo!(), //Box::new(Text::new("Root")),
                    SnowcapNodeData::Container => {
                        debug!("Container");
                        SnowcapContainer::new(
                            attrs.unwrap_or(Attributes::default()),
                            Self::content_single(*node_id, contents)?.ok_or(
                                ConversionError::Missing("expecting container content".into()),
                            )?,
                        )?
                    }
                    SnowcapNodeData::Widget(label) => {
                        debug!("Widget({label})");

                        SnowcapWidget::new(
                            *node_id,
                            label.clone(),
                            data.element_id.clone(),
                            attrs.unwrap_or(Attributes::default()),
                            contents,
                        )?
                        .with_node_id(*node_id)
                    }
                    SnowcapNodeData::Row => {
                        let num_children = children.as_ref().map(|children| children.len());
                        debug!("Row [children={num_children:?}]");
                        SnowcapRow::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(*node_id)
                    }
                    SnowcapNodeData::Column => {
                        let num_children = children.as_ref().map(|children| children.len());
                        debug!("Column [children={num_children:?}]");
                        SnowcapColumn::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(*node_id)
                    }
                    SnowcapNodeData::Stack => {
                        let num_children = contents.as_ref().map(|children| children.len());
                        debug!("Stack [children={num_children:?}]");
                        SnowcapStack::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(*node_id)
                    }
                    SnowcapNodeData::Value(_value) => {
                        info!("VALUE");
                        todo!()
                    } //Box::new(Text::new("Value")),
                };

                Ok(widget)
            });

            widget
        })
    }

    /*
    pub fn from_node(node: NodeRef<M>) -> Result<DynamicWidget<'static, M>, ConversionError> {
        trace_span!("from-node").in_scope(|| {
            let widget = node.with_data(|data| {
                let node = node.node();
                let content = node.children();

                let attrs = data.attrs.clone();

                let widget: Box<dyn iced::advanced::Widget<M, iced::Theme, iced::Renderer>> =
                    match &data.data {
                        SnowcapNodeData::None => todo!(),
                        SnowcapNodeData::Root => Box::new(Text::new("Root")),
                        SnowcapNodeData::Container => {
                            if let Some(content) = content {
                                let content = content
                                    .first()
                                    .ok_or(ConversionError::Missing("content".into()))?;

                                /*
                                Box::new(SnowcapContainer::new(
                                    attrs.unwrap_or(Attributes::default()),
                                    content.clone(),
                                )?)
                                */

                                Box::new(Text::new("none"))
                            } else {
                                return Err(ConversionError::Missing("content".into()));
                            }
                        }
                        SnowcapNodeData::Widget(label) => {
                            let content = if let Some(content) = content {
                                let content = content
                                    .first()
                                    .ok_or(ConversionError::Missing("content".into()))?;

                                Some(content.clone())
                            } else {
                                None
                            };

                            SnowcapWidget::<M>::new(
                                node.id().clone(),
                                label.clone(),
                                data.element_id.clone(),
                                attrs.unwrap_or(Attributes::default()),
                                None,
                            )?
                        }
                        SnowcapNodeData::Row => {
                            let contents =
                                content.ok_or(ConversionError::Missing("content".into()))?;

                            Box::new(SnowcapRow::convert(
                                attrs.unwrap_or(Attributes::default()),
                                &*contents,
                            )?)
                        }
                        SnowcapNodeData::Column => {
                            let contents =
                                content.ok_or(ConversionError::Missing("content".into()))?;

                            Box::new(SnowcapColumn::convert(
                                attrs.unwrap_or(Attributes::default()),
                                &*contents,
                            )?)
                        }
                        SnowcapNodeData::Stack => todo!(),
                        SnowcapNodeData::Value(_value) => Box::new(Text::new("Value")),
                    };

                Ok(widget)
            })?;

            Ok(DynamicWidget::from(widget))
        })
    }
    */
}
