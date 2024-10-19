use std::{collections::HashMap, ops::Deref, sync::Arc, time::Duration};

use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLock};
use strum::{EnumDiscriminants, EnumIter};

use crate::SyncError;

#[derive(Debug, Clone, EnumDiscriminants, PartialEq)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(AttributeKind))]
pub enum AttributeValue {
    TextColor(iced::Color),
    Border(iced::Border),
    Shadow(iced::Shadow),
    HorizontalAlignment(iced::alignment::Horizontal),
    VerticalAlignment(iced::alignment::Vertical),
    Padding(iced::Padding),
    WidthLength(iced::Length),
    WidthPixels(iced::Pixels),
    MaxWidth(iced::Pixels),
    HeightLength(iced::Length),
    HeightPixels(iced::Pixels),
    Background(iced::Background),
    Spacing(iced::Pixels),
    Size(iced::Pixels),
    CellSize(iced::Pixels),
    Clip(bool),
    Toggled(bool),
    Selected(String),
    Label(String),
    Theme(iced::Theme),
    Wrapping(iced::widget::text::Wrapping),
    Shaping(iced::widget::text::Shaping),
}

impl AttributeValue {
    /// Get the discriminant of this [`AttributeValue`], returning an [`AttributeKind`]
    pub fn kind(&self) -> AttributeKind {
        self.into()
    }
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attribute_type: AttributeKind = self.into();
        write!(f, "{:?}", attribute_type)
    }
}

/// A set of ['Attribute']. This is represented as a ['HashMap'] wrapped in an ['Arc'] and ['parking_lot::RwLock']
/// allowing the attributes to be cloned, and sent between threads
#[derive(Default, Clone)]
pub struct Attributes(Arc<RwLock<HashMap<AttributeKind, Attribute>>>);

impl Attributes {
    /// Create an empty set of attributes
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of attributes in the set
    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    /// Try to acquire a write lock, and push the [`Attribute`] to the set of [`Attributes`]
    pub fn push(&mut self, attr: Attribute) -> Result<(), SyncError> {
        if let Some(mut guard) = self.0.try_write() {
            guard.insert(attr.kind(), attr);
            Ok(())
        } else {
            Err(SyncError::Deadlock(format!(
                "RwLock try_write() failed in Attributes::push() {:#?}",
                attr
            )))
        }
    }

    /// Get an [`AttributeValue`] from the set of [`Attributes'] for the specified [`AttributeKind`]
    pub fn get(&self, kind: AttributeKind) -> Result<Option<AttributeValue>, SyncError> {
        match self.0.try_read_for(Duration::from_secs(1)) {
            Some(guard) => Ok(guard.get(&kind).map(|attr| attr.value().clone())),
            None => Err(SyncError::Deadlock(format!(
                "RwLock read lock failed in Attributes::get() {:?}",
                kind
            ))),
        }
    }

    pub fn set(&self, value: AttributeValue) -> Result<(), SyncError> {
        match self.0.try_write_for(Duration::from_secs(1)) {
            Some(mut guard) => {
                guard.insert(value.kind(), Attribute::new(value));
                Ok(())
            }
            None => Err(SyncError::Deadlock(format!(
                "RwLock write lock failed in Attributes::set() {:?}",
                value
            ))),
        }
    }
}

impl std::fmt::Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        let mut iter = self.into_iter().peekable();
        loop {
            if let Some(attr) = iter.next() {
                write!(f, "{:?}", attr.value())?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            } else {
                break;
            }
        }
        f.write_str("]")?;

        Ok(())
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = Attribute;

    type IntoIter = AttributeIter;

    fn into_iter(self) -> Self::IntoIter {
        // Acquire an ArcRwLockReadGuard. This has a static lifetime that we can use with Iterator
        match self.0.try_read_arc_for(Duration::from_secs(1)) {
            Some(guard) => {
                // Collect a vec of AttributeKinds for each key in the HashMap. The iterator will iterate through
                // each kind from the set to yield a reference to the [`Attribute`] while the RwLock Arc read guard is held
                let attr_kinds: Vec<AttributeKind> =
                    guard.iter().map(|(kind, _attr)| *kind).collect();

                AttributeIter {
                    guard,
                    iter: attr_kinds.into_iter(),
                }
            }
            None => {
                panic!("Deadlock in Attributes iterator");
            }
        }
    }
}

impl IntoIterator for Attributes {
    type Item = Attribute;

    type IntoIter = AttributeIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self).into_iter()
    }
}

pub struct AttributeIter {
    guard: ArcRwLockReadGuard<RawRwLock, HashMap<AttributeKind, Attribute>>,
    //attr_kinds: Vec<AttributeKind>,
    iter: std::vec::IntoIter<AttributeKind>,
}

impl Iterator for AttributeIter {
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(kind) = &self.iter.next() {
            self.guard.get(kind).map(|item| item.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    value: AttributeValue,
}

impl Deref for Attribute {
    type Target = AttributeValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Attribute {
    /// Return a reference to the inner [`AttributeValue`'] for this Attribute
    pub fn value<'a>(&'a self) -> &'a AttributeValue {
        &self.value
    }

    /// Return a mutable reference to the inner [`AttributeValue`'] for this Attribute
    pub fn value_mut<'a>(&'a mut self) -> &'a mut AttributeValue {
        &mut self.value
    }

    /// Create a new [`Attribute`] with the provided [`AttributeValue`]
    pub fn new(value: AttributeValue) -> Self {
        Self { value }
    }
}

/// Hash the discriminant of the inner AttributeValue of this Attribute
impl std::hash::Hash for Attribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().kind().hash(state);
    }
}

/// Test for equality of the discriminant of the inner [`AttributeValue`] of this [`Attribute`]
impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl Eq for Attribute {}

#[cfg(test)]
mod attribute_tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_attribute() {
        let mut attrs = Attributes::new();

        let attr = Attribute::new(AttributeValue::Clip(true));
        attrs.push(attr).unwrap();
        assert!(attrs.len() == 1);

        // Pushing the same attribute should not change the length
        let attr = Attribute::new(AttributeValue::Clip(true));
        attrs.push(attr).unwrap();
        assert!(attrs.len() == 1);

        for attr in attrs {
            assert!(attr.kind() == AttributeKind::Clip)
        }
    }
}
