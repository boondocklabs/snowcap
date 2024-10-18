use arbutus::Node as _;
use arbutus::NodeRef as _;
use iced::widget::Text;
use tracing::{info, trace_span};

use crate::{
    attribute::Attributes, message::WidgetMessage, node::SnowcapNodeData, widget::WidgetWrap,
    ConversionError, DynamicWidget, NodeId, NodeRef,
};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, widget::SnowcapWidget,
};

impl<M> DynamicWidget<M>
where
    M: Clone + std::fmt::Debug + From<(NodeId, WidgetMessage)> + 'static,
{
    pub fn from_node(mut node: NodeRef<M>) -> Result<DynamicWidget<M>, ConversionError> {
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

                                Box::new(SnowcapContainer::new(
                                    attrs.unwrap_or(Attributes::default()),
                                    content.clone(),
                                )?)
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

                            SnowcapWidget::<M>::from_tree_node(
                                node.id().clone(),
                                label.clone(),
                                data.element_id.clone(),
                                attrs.unwrap_or(Attributes::default()),
                                content,
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

            /*
            let _: Result<(), ()> = node.with_data_mut(|inner| {
                let mut inner = inner;
                if let Some(x) = &mut inner.widget {
                    tracing::info!("REPLACING WIDGET IN NODE");
                    x.replace(widget);
                } else {
                    tracing::info!("SET FIRST WIDGET IN NODE");
                    inner.widget = Some(WidgetWrap::new(widget))
                }

                Ok(())
            });

            info!("<---- From Node {}", node.node().id());

            Ok(DynamicWidget::from(Box::new(
                node.node().data().widget.as_ref().unwrap().widget(), //WidgetWrap::new(widget).widget(),
            )))
            */

            Ok(DynamicWidget::from(widget))
        })
    }
}
