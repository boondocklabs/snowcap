use iced::{advanced::Widget, Element};

pub struct DynamicWidget<M> {
    widget: Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'static>,
}

impl<M> DynamicWidget<M> {
    pub fn new(widget: impl Widget<M, iced::Theme, iced::Renderer> + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn from(widget: Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'static>) -> Self {
        Self { widget }
    }

    pub fn into_element(self) -> Element<'static, M>
    where
        M: 'static,
    {
        Element::new(self)
    }
}

/// Implementation of iced Widget trait on SnowcapWidget
impl<M> Widget<M, iced::Theme, iced::Renderer> for DynamicWidget<M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget.diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget.size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget.size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget.operate(tree, layout, renderer, operation);
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
        self.widget.on_event(
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
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<M, iced::Theme, iced::Renderer>> {
        self.widget.overlay(tree, layout, renderer, translation)
    }
}

/// Implement Widget on a mutable reference to a DynamicWidget
impl<M> Widget<M, iced::Theme, iced::Renderer> for &mut DynamicWidget<M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget.diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget.size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget.size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget.operate(tree, layout, renderer, operation);
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
        self.widget.on_event(
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
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<M, iced::Theme, iced::Renderer>> {
        self.widget.overlay(tree, layout, renderer, translation)
    }
}
