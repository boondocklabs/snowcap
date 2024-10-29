use std::{sync::Arc, time::Duration};

use colored::Colorize as _;
use iced::{advanced::Widget, Element};
use parking_lot::{ArcRwLockWriteGuard, RawRwLock, RwLock};
use tracing::{debug, warn};

use crate::{NodeId, SyncError};

/// Widget reference is an Arc RwLockWriteGuard that can be acquired from a clone of [`DynamicWidget`]
/// and wrapped into an `iced::Element<'static>`. This enables reuse of the same underlying widget.
///
/// Only one `WidgetRef` may be acquired at a time from a [`DynamicWidget`].
pub struct WidgetRef<M> {
    node_id: NodeId,
    widget: ArcRwLockWriteGuard<RawRwLock, Box<dyn Widget<M, iced::Theme, iced::Renderer>>>,
}

impl<M> Drop for WidgetRef<M> {
    fn drop(&mut self) {
        debug!("{} {}", "WidgetRef dropped".bright_magenta(), self.node_id);
    }
}

/// Wraps a dyn Widget in an Arc<parking_lot::RwLock>, allowing the widget to be cloned and converted to an `iced::Element` by reference
/// with a 'static lifetime. When converted to an Element, the guard will be held in a [`WidgetRef`] until the Element is dropped,
/// but the DynamicWidget itself and the underlying iced Widget will remain and can be re-acquired on subsequent view() calls.
pub struct DynamicWidget<M> {
    node_id: Option<NodeId>,
    widget: Option<Arc<RwLock<Box<dyn Widget<M, iced::Theme, iced::Renderer>>>>>,
}

impl<M> Clone for DynamicWidget<M> {
    fn clone(&self) -> Self {
        tracing::debug!("Cloning DynamicWidget {:?}", self.node_id);
        DynamicWidget {
            node_id: self.node_id,
            widget: self.widget.clone(),
        }
    }
}

impl<'a, M> std::default::Default for DynamicWidget<M> {
    fn default() -> Self {
        Self {
            node_id: None,
            widget: None,
        }
    }
}

impl<M> TryInto<Element<'_, M>> for DynamicWidget<M>
where
    M: 'static,
{
    type Error = crate::SyncError;

    fn try_into(self) -> Result<Element<'static, M>, Self::Error> {
        let lock = self.widget.unwrap();
        let guard = lock
            .try_write_arc_for(Duration::from_secs(1))
            .ok_or(SyncError::Deadlock(format!(
                "DynamicWidget already locked. Node {:?}",
                self.node_id
            )))?;
        let widget_ref = WidgetRef {
            widget: guard,
            node_id: self.node_id.unwrap(),
        };
        debug!("New WidgetRef node {:?}", self.node_id);
        Ok(Element::new(widget_ref))
    }
}

impl<M> std::fmt::Debug for DynamicWidget<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DynamicWidget node_id={:?}",
            self.node_id.unwrap_or(9999999)
        )
    }
}

impl<M> DynamicWidget<M> {
    pub fn from(widget: impl Widget<M, iced::Theme, iced::Renderer> + 'static) -> Self {
        Self {
            node_id: None,
            widget: Some(Arc::new(RwLock::new(Box::new(widget)))),
        }
    }

    pub fn into_inner(self) -> Result<Box<dyn Widget<M, iced::Theme, iced::Renderer>>, SyncError> {
        if let Some(widget) = self.widget {
            match Arc::try_unwrap(widget) {
                Ok(lock) => Ok(lock.into_inner()),
                Err(_) => Err(SyncError::Deadlock(
                    "Cannot get inner Widget out of Arc if DynamicWidget has been cloned".into(),
                )),
            }
        } else {
            panic!("No inner widget");
        }
    }

    pub fn with_widget(
        mut self,
        widget: impl Widget<M, iced::Theme, iced::Renderer> + 'static,
    ) -> Self {
        self.widget = Some(Arc::new(RwLock::new(Box::new(widget))));
        self
    }

    pub fn with_node_id(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    /// Replace the inner Boxed dyn Widget. This requires there are no [`WidgetRef`] alive, as they hold a write lock
    pub fn replace(
        &self,
        widget: Box<dyn Widget<M, iced::Theme, iced::Renderer>>,
    ) -> Result<(), SyncError> {
        if let Some(inner) = &self.widget {
            debug!("Replacing widget for node {:?}", self.node_id);
            *inner.try_write().ok_or(SyncError::Deadlock(
                "DynamicWidget::replace() cannot replace Widget while write lock is held".into(),
            ))? = widget;
            Ok(())
        } else {
            warn!("Attempt to replace widget, but no existing widget set");
            Ok(())
        }
    }

    pub fn into_element(self) -> Result<Element<'static, M>, SyncError>
    where
        M: 'static,
    {
        self.try_into()
    }
}

/// Implementation of iced Widget trait on WidgetRef, which holds
/// a Write lock on a DynamicWidget's Boxed dyn Widget
#[profiling::all_functions]
impl<M> Widget<M, iced::Theme, iced::Renderer> for WidgetRef<M> {
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
