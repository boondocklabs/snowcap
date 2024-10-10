use iced::{
    widget::{Space, Text},
    Element,
};
use tracing::info;

use crate::{message::WidgetMessage, tree::node::TreeNode, ConversionError, MarkupTreeNode};

use super::{
    column::SnowcapColumn, container::SnowcapContainer, row::SnowcapRow, stack::SnowcapStack,
    widget::SnowcapWidget,
};

impl<'a, M> SnowcapWidget<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn from_node(
        node: MarkupTreeNode<'a, M>,
    ) -> Result<Option<SnowcapWidget<'a, M>>, ConversionError> {
        let widget: Box<dyn iced::advanced::Widget<M, iced::Theme, iced::Renderer> + 'a> =
            match node.clone() {
                MarkupTreeNode::None => Box::new(Space::new(0, 0)),
                MarkupTreeNode::Container { attrs, content, .. } => {
                    info!("CONTAINER");
                    Box::new(SnowcapContainer::new(attrs, content)?)
                }
                MarkupTreeNode::Widget {
                    name,
                    attrs,
                    content,
                    ..
                } => {
                    info!("WIDGET");
                    SnowcapWidget::from_tree_node(name, attrs, content)?
                }
                MarkupTreeNode::Row {
                    element_id,
                    attrs,
                    contents,
                } => {
                    info!("ROW {contents:#?}");
                    Box::new(SnowcapRow::convert(attrs, contents)?)
                }
                MarkupTreeNode::Column {
                    element_id,
                    attrs,
                    contents,
                } => Box::new(SnowcapColumn::convert(attrs, contents)?),
                MarkupTreeNode::Stack {
                    element_id,
                    attrs,
                    contents,
                } => Box::new(SnowcapStack::convert(attrs, contents)?),
                MarkupTreeNode::Label(_) => return Ok(None),
                MarkupTreeNode::Value(value) => {
                    info!("VALUE");
                    match &*value.borrow() {
                        crate::Value::String(s) => Box::new(Text::new(s.clone())),
                        crate::Value::Number(n) => Box::new(Text::new(n.clone())),
                        crate::Value::Boolean(b) => Box::new(Text::new(b.clone())),
                        crate::Value::Array(vec) => return Ok(None),
                        crate::Value::Data { data, provider } => {
                            info!("Data");
                            if let Some(data) = data {
                                info!("Have data");
                                /*
                                match &**data {
                                    crate::data::DataType::Null => todo!(),
                                    crate::data::DataType::Image(handle) => todo!(),
                                    crate::data::DataType::Svg(handle) => todo!(),
                                    crate::data::DataType::QrCode(arc) => todo!(),
                                    crate::data::DataType::Markdown(markdown_items) => todo!(),
                                    crate::data::DataType::Text(_) => todo!(),
                                }
                                */
                                Box::new(Text::new("SOME DATA"))
                            } else {
                                info!("No data");
                                Box::new(Text::new("NO DATA"))
                            }
                        }
                    }
                }
                MarkupTreeNode::Phantom(_) => return Ok(None),
            };

        Ok(Some(SnowcapWidget::new(node, widget)))
    }
}

impl<'a, M> MarkupTreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + 'a + From<WidgetMessage>,
{
    pub fn into_widget(self) -> Result<SnowcapWidget<'a, M>, ConversionError>
    where
        M: Clone + std::fmt::Debug + From<M> + 'a,
    {
        if let Some(widget) = SnowcapWidget::from_node(self)? {
            Ok(widget)
        } else {
            Err(ConversionError::Missing("No widget".into()))
        }
    }
}

impl<'a, M> TreeNode<'a, M>
where
    M: Clone + std::fmt::Debug + From<WidgetMessage> + 'a,
{
    pub fn into_element(mut self) -> Result<Element<'a, M>, ConversionError> {
        match self.widget.take() {
            Some(widget) => Ok(Element::new(*widget)),
            None => {
                tracing::error!("No widget for node {self:#?}");
                Err(ConversionError::Missing("Widget".into()))
            }
        }
    }
}
