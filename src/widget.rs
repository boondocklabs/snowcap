/*
use strum::{EnumDiscriminants, EnumIter};

#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum WidgetType<'a, M> {
    Button(iced::widget::Button<'a, M>),
    //Canvas(iced::widget::Canvas),
    Checkbox(iced::widget::Checkbox<'a, M>),
    Column(iced::widget::Column<'a, M>),
    //ComboBox(iced::widget::ComboBox<'a, (), M>),
    Container(iced::widget::Container<'a, M>),
    Image(iced::widget::Image),
    MouseArea(iced::widget::MouseArea<'a, M>),
    PaneGrid(iced::widget::PaneGrid<'a, M>),
    //PickList(iced::widget::PickList<'a, (), (), (), M>),
    Text(iced::widget::Text<'a>),
}
*/

use std::sync::{atomic::AtomicU64, Arc};

use iced::advanced::Widget;
use parking_lot::{ArcRwLockUpgradableReadGuard, RawRwLock, RwLock};
use tracing::info;

pub struct WidgetWrap<M> {
    node: Arc<RwLock<Box<dyn Widget<M, iced::Theme, iced::Renderer>>>>,
    version: Arc<AtomicU64>,
}

impl<M> std::fmt::Debug for WidgetWrap<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WidgetWrap")
    }
}

impl<M> WidgetWrap<M> {
    pub fn new(node: Box<dyn Widget<M, iced::Theme, iced::Renderer>>) -> Self {
        info!("NEW WIDGET WRAPPED");
        Self {
            node: Arc::new(RwLock::new(node)),
            version: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn widget(&self) -> WidgetRef<M>
    where
        M: 'static,
    {
        info!("Issue new WidgetRef. Version {:?}", self.version);
        let guard = self.node.try_upgradable_read_arc().unwrap();
        WidgetRef::new(guard, self.version.clone())
    }

    pub fn replace(&mut self, new: Box<dyn Widget<M, iced::Theme, iced::Renderer>>) {
        *self.node.write() = new;
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        info!("Widget replaced. Version {:?}", self.version);
    }
}

pub struct WidgetRef<M> {
    widget: ArcRwLockUpgradableReadGuard<
        RawRwLock,
        Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'static>,
    >,
    version: Arc<AtomicU64>,
}

impl<M> WidgetRef<M>
where
    M: 'static,
{
    pub fn new(
        guard: ArcRwLockUpgradableReadGuard<
            RawRwLock,
            Box<dyn Widget<M, iced::Theme, iced::Renderer> + 'static>,
        >,
        version: Arc<AtomicU64>,
    ) -> Self {
        info!("NEW WIDGET REF CREATED");
        Self {
            widget: guard,
            version,
        }
    }
}

/// Implement Widget on a mutable reference to a DynamicWidget
impl<'a, M> Widget<M, iced::Theme, iced::Renderer> for WidgetRef<M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        info!("Proxy state");
        self.widget.state()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.version.load(std::sync::atomic::Ordering::SeqCst);
        info!("Proxy diff version {:?}", self.version);
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
        self.widget.with_upgraded(|w| {
            w.on_event(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            )
        })
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
        /*
        self.widget.with_upgraded(
            |widget: &'b mut Box<dyn Widget<M, iced::Theme, iced::Renderer>>| {
                widget.overlay(tree, layout, renderer, translation)
            },
        )
        */
        None
    }
}
