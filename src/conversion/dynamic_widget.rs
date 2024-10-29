use arbutus::TreeNode as _;
use arbutus::TreeNodeRef as _;
use tracing::debug;
use tracing::debug_span;
use tracing::info;

use crate::conversion::stack::SnowcapStack;
use crate::parser::value::ValueKind;
use crate::tree_util::WidgetContent;
use crate::{
    attribute::Attributes, message::WidgetMessage, node::SnowcapNodeData, ConversionError,
    DynamicWidget, NodeId, NodeRef,
};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, widget::SnowcapWidget,
};

impl<'a, M> DynamicWidget<M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    /*
    fn content_single<'b>(
        children: Option<&'b Vec<WidgetContent<'a, M>>>,
        //) -> Result<Option<DynamicWidget<'a, M>>, ConversionError> {
    ) -> Option<&'b WidgetRef<'a, M>> {
        let child = children?.first()?;
        match child {
            WidgetContent::Widget(dynamic_widget) => Some(dynamic_widget),
            _ => todo!(), /*
                          ChildData::Value(value) => match &**value {
                              ValueKind::String(_) => todo!(),
                              ValueKind::Float(_) => todo!(),
                              ValueKind::Integer(_) => todo!(),
                              ValueKind::Boolean(_) => todo!(),
                              ValueKind::Array(_vec) => todo!(),
                              ValueKind::Dynamic {
                                  data: _,
                                  provider: _,
                              } => {
                                  todo!()
                                  //Ok(Some(SnowcapWidget::loading()))
                              }
                          },
                          */
        }
    }
    */

    pub fn builder(
        node: NodeRef<M>,
        content: WidgetContent<M>,
    ) -> Result<DynamicWidget<M>, ConversionError> {
        debug_span!("DynamicWidget").in_scope(|| {
            debug!("Building node_id={:?}", node.node().id());

            Ok(SnowcapWidget::loading())

            /*
            let widget = node.with_data(|data| {
                let node = node.node();
                let node_id = node.id();
                let attrs = data.attrs.clone();

                // Collect the contents in the order specified in the node
                //
                /*
                let contents = children.as_mut().map(|children| {
                    let contents: Option<Vec<ChildData<M>>> = node.children().map(|child| {
                        child
                            .iter()
                            .map(|f| children.remove(&f.node().id()).unwrap())
                            .collect()
                    });
                    contents.unwrap()
                });
                */

                let contents = children;

                let widget = match &data.data {
                    SnowcapNodeData::None => todo!(),
                    SnowcapNodeData::Root => todo!(), //Box::new(Text::new("Root")),
                    SnowcapNodeData::Container => {
                        debug!("Container");
                        SnowcapContainer::new(
                            attrs.unwrap_or(Attributes::default()),
                            Self::content_single(node_id, contents)?.ok_or(
                                ConversionError::Missing("expecting container content".into()),
                            )?,
                        )?
                    }
                    SnowcapNodeData::Widget(label) => {
                        debug!("Widget({label})");

                        SnowcapWidget::new(
                            node_id,
                            label.clone(),
                            data.element_id.clone(),
                            attrs.unwrap_or(Attributes::default()),
                            contents,
                        )?
                        .with_node_id(node_id)
                    }
                    SnowcapNodeData::Row => {
                        let num_children = children.as_ref().map(|children| children.len());
                        debug!("Row [children={num_children:?}]");
                        SnowcapRow::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(node_id)
                    }
                    SnowcapNodeData::Column => {
                        let num_children = children.as_ref().map(|children| children.len());
                        debug!("Column [children={num_children:?}]");
                        SnowcapColumn::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(node_id)
                    }
                    SnowcapNodeData::Stack => {
                        let num_children = contents.as_ref().map(|children| children.len());
                        debug!("Stack [children={num_children:?}]");
                        SnowcapStack::convert(attrs.unwrap_or(Attributes::default()), contents)?
                            .with_node_id(node_id)
                    }
                    SnowcapNodeData::Value(_value) => {
                        info!("VALUE");
                        todo!()
                    } //Box::new(Text::new("Value")),
                };

                Ok(widget)
            });

            widget
            */
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
