use std::marker::PhantomData;

use iced::{advanced::Widget, Element};

use crate::{tree_util::ChildData, NodeId};

pub struct DynamicWidget<'a, M> {
    node_id: Option<NodeId>,
    widget: Option<Box<dyn Widget<M, iced::Theme, iced::Renderer>>>,
    children: Option<Vec<ChildData<'a, M>>>,
    _phantom: PhantomData<&'a M>,
}

impl<'a, M> std::default::Default for DynamicWidget<'a, M> {
    fn default() -> Self {
        Self {
            node_id: None,
            widget: None,
            children: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, M> Into<Element<'a, M>> for DynamicWidget<'a, M>
where
    M: 'a,
{
    fn into(self) -> Element<'a, M> {
        Element::new(self)
    }
}

impl<'a, M> Into<Element<'a, M>> for Box<&'a mut DynamicWidget<'a, M>>
where
    M: 'a,
{
    fn into(self) -> Element<'a, M> {
        Element::new(self)
    }
}

impl<'a, M> std::fmt::Debug for DynamicWidget<'a, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DynamicWidget node_id={:?}",
            self.node_id.unwrap_or(9999999)
        )
    }
}

impl<'a, M> DynamicWidget<'a, M> {
    pub fn from(widget: Box<dyn Widget<M, iced::Theme, iced::Renderer>>) -> Self {
        Self {
            node_id: None,
            widget: Some(widget),
            children: None,
            _phantom: PhantomData,
        }
    }
    pub fn with_children(mut self, children: Option<Vec<ChildData<'a, M>>>) -> Self {
        self.children = children;
        self
    }

    pub fn with_widget(
        mut self,
        widget: impl Widget<M, iced::Theme, iced::Renderer> + 'static,
    ) -> Self {
        self.widget = Some(Box::new(widget));
        self
    }

    pub fn with_node_id(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    // Get a vec of references to content widgets
    pub fn contents(&mut self) -> Option<Vec<Box<&mut Self>>> {
        self.children.as_mut().map(|children| {
            children
                .iter_mut()
                .filter_map(|child| {
                    if let ChildData::Widget(widget) = child {
                        Some(Box::new(widget))
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    pub fn into_element(self) -> Element<'a, M>
    where
        M: 'a,
    {
        Element::new(self)
    }
}

/// Implementation of iced Widget trait on SnowcapWidget
impl<'a, M> Widget<M, iced::Theme, iced::Renderer> for DynamicWidget<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.as_ref().unwrap().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget.as_ref().unwrap().state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.as_ref().unwrap().children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget.as_ref().unwrap().diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget.as_ref().unwrap().size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget.as_ref().unwrap().size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget.as_ref().unwrap().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget
            .as_ref()
            .unwrap()
            .operate(tree, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        self.widget.as_mut().unwrap().on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.widget
            .as_ref()
            .unwrap()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.widget
            .as_ref()
            .unwrap()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<M, iced::Theme, iced::Renderer>> {
        self.widget
            .as_mut()
            .unwrap()
            .overlay(tree, layout, renderer, translation)
    }
}

/// Implement Widget on a mutable reference to a DynamicWidget
impl<'a, M> Widget<M, iced::Theme, iced::Renderer> for Box<&mut DynamicWidget<'a, M>> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.as_ref().unwrap().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget.as_ref().unwrap().state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.as_ref().unwrap().children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget.as_ref().unwrap().diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget.as_ref().unwrap().size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget.as_ref().unwrap().size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget.as_ref().unwrap().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget
            .as_ref()
            .unwrap()
            .operate(tree, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        self.widget.as_mut().unwrap().on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.widget
            .as_ref()
            .unwrap()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.widget
            .as_ref()
            .unwrap()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<M, iced::Theme, iced::Renderer>> {
        self.widget
            .as_mut()
            .unwrap()
            .overlay(tree, layout, renderer, translation)
    }
}
