use iced::{advanced::Widget, Element};

/// Wrap an element with Widget impl, so it can be used as a dyn Widget.
pub struct ElementWrapper<M> {
    element: Element<'static, M>,
}

impl<M> ElementWrapper<M> {
    pub fn new(element: Element<'static, M>) -> Self {
        Self { element }
    }

    fn widget(&self) -> &dyn Widget<M, iced::Theme, iced::Renderer> {
        self.element.as_widget()
    }

    fn widget_mut(&mut self) -> &mut dyn Widget<M, iced::Theme, iced::Renderer> {
        self.element.as_widget_mut()
    }
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for ElementWrapper<M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget().tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.widget().state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget().children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.widget().diff(tree);
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.widget().size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.widget().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.widget().operate(tree, layout, renderer, operation);
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
        self.widget_mut().on_event(
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
        self.widget()
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
        self.widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.widget_mut()
            .overlay(tree, layout, renderer, translation)
    }
}
