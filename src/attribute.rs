//! Widget Attributes, parsed from snowcap grammar using [`crate::parser::attribute::AttributeParser`]
//! using Pest grammar [`src/parser/attribute.pest`](https://github.com/boondocklabs/snowcap/blob/main/src/parser/attribute.pest)

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
    time::Duration,
};

use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLock};
use strum::{EnumDiscriminants, EnumIter};
use xxhash_rust::xxh64::Xxh64;

use crate::SyncError;

mod hash;

/// All possible [`Attribute`] inner values
#[derive(Debug, Clone, EnumDiscriminants, PartialEq)]
#[strum_discriminants(derive(EnumIter, Hash, PartialOrd, Ord))]
#[strum_discriminants(name(AttributeKind))]
pub enum AttributeValue {
    /// Text Color sRGB color space
    TextColor(iced::Color),
    /// Border which can be applied to styles
    Border(iced::Border),
    /// Shadow which can be applied to styles
    Shadow(iced::Shadow),
    /// Horizontal alignment
    HorizontalAlignment(iced::alignment::Horizontal),
    /// Vertical alignment
    VerticalAlignment(iced::alignment::Vertical),
    /// Padding. Can be all sides, top/bottom, left/right, or individually
    Padding(iced::Padding),
    /// Width in units of [`iced::Length`]
    WidthLength(iced::Length),
    /// Width in units of [`iced::Pixels`]
    WidthPixels(iced::Pixels),
    /// Maximum width in [`iced::Pixels`]
    MaxWidth(iced::Pixels),
    /// Height in units of [`iced::Length`]
    HeightLength(iced::Length),
    /// Height in units of [`iced::Pixels`]
    HeightPixels(iced::Pixels),
    /// Background of an element. Color or Gradient.
    Background(iced::Background),
    /// Spacing between elements
    Spacing(iced::Pixels),
    /// Size in [`iced::Pixels`]
    Size(iced::Pixels),
    /// QR Code Cell Size
    CellSize(iced::Pixels),
    /// Clipping flag
    Clip(bool),
    /// Toggled flag for toggle widget
    Toggled(bool),
    /// Selected value for pick list widget
    Selected(String),
    /// A label
    Label(String),
    /// Built in [`iced::Theme`]
    Theme(iced::Theme),
    /// Text wrapping
    Wrapping(iced::widget::text::Wrapping),
    /// Text shaping
    Shaping(iced::widget::text::Shaping),
    /// Slider Value
    SliderValue(i32),
    /// Scroll Direction
    ScrollDirection(iced::widget::scrollable::Direction),
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

/// A set of [`Attribute`] items. This is represented as a [`HashMap`] wrapped in an [`Arc`] and [`parking_lot::RwLock`]
/// allowing the attributes to be cloned, and sent between threads
#[derive(Default, Clone)]
pub struct Attributes(Arc<RwLock<HashMap<AttributeKind, Attribute>>>);

impl std::hash::Hash for Attributes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.each_with(state, |state, attr| {
            attr.hash(state);
        })
        .unwrap();
    }
}

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
    pub fn push(&mut self, attr: Attribute) -> Result<&Self, SyncError> {
        if let Some(mut guard) = self.0.try_write() {
            guard.insert(attr.kind(), attr);
            Ok(self)
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

    pub fn each_with<T, F>(&self, mut with: T, f: F) -> Result<T, SyncError>
    where
        F: Fn(&mut T, &Attribute),
    {
        match self.0.try_read_for(Duration::from_secs(1)) {
            Some(guard) => {
                // Collect and sort keys from the HashMap to yield attributes
                // in a deterministic order
                let mut keys: Vec<&AttributeKind> = guard.keys().collect();
                keys.sort();

                for key in keys {
                    if let Some(attr) = guard.get(key) {
                        f(&mut with, attr);
                    }
                }

                Ok(with)
            }
            None => Err(SyncError::Deadlock(format!(
                "RwLock read lock failed in Attributes::each()",
            ))),
        }
    }

    /// Get the Xxh64 hash of the set of attributes
    pub fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);

        hasher = self
            .each_with(hasher, |hasher, attr| {
                attr.hash(hasher);
            })
            .unwrap();

        hasher.finish()
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

/// Convert a reference to [`Attributes`] into an [`AttributeIter`]
/// This attempts to obtain an Arc read lock guard, which has a `static lifetime
/// so the guard can be included in the iterator state. This holds the read lock
/// while the iterator is in scope.
///
/// Each attribute discriminant present in the set of attributes is collected
/// into a vector, and the iterator then steps through the discriminants to
/// obtain each variant from the HashMap.
///
/// It is required that the iterator order is deterministic for hashing
impl<'a> IntoIterator for &'a Attributes {
    type Item = Attribute;

    type IntoIter = AttributeIter;

    fn into_iter(self) -> Self::IntoIter {
        // Acquire an ArcRwLockReadGuard. This has a static lifetime that we can use with Iterator
        match self.0.try_read_arc_for(Duration::from_secs(1)) {
            Some(guard) => {
                // Collect a vec of AttributeKinds for each key in the HashMap. The iterator will iterate through
                // each kind from the set to yield a reference to the [`Attribute`] while the RwLock Arc read guard is held
                let mut attr_kinds: Vec<AttributeKind> =
                    guard.iter().map(|(kind, _attr)| *kind).collect();

                // Ensure the vec of attributes is sorted, so the attributes are yielded in a deterministic order.
                // This is required for producing deterministic hashes of attribute sets.
                attr_kinds.sort();

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

/// An [`Iterator`] over [`Attribute`] items
pub struct AttributeIter {
    guard: ArcRwLockReadGuard<RawRwLock, HashMap<AttributeKind, Attribute>>,
    iter: std::vec::IntoIter<AttributeKind>,
}

impl<'a> Iterator for AttributeIter {
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(kind) = &self.iter.next() {
            self.guard.get(kind).map(|item| item.clone())
        } else {
            None
        }
    }
}

/// Attribute represents an attribute value which has been parsed from the
/// markup and converted into a concrete iced/rust type.
///
/// This struct wraps the [`AttributeValue`] enum which defines
/// a variant for each possible attribute that can appear in the markup.
///
/// An Attribute is typically stored in an [`Attributes`] container, which
/// maintains a set of Attribute values belonging to a Widget.
///
/// An Attribute may be hashed, and will include the encapsulated data
/// for tree diffing to detect changes in attribute values.
#[derive(Debug, Clone, Hash)]
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

    /// Get the Xxh64 hash of this attribute, including the inner values
    pub fn xxhash(&self) -> u64 {
        let mut hasher = Xxh64::new(0);
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod attribute_tests {

    use crate::parser::attribute::AttributeParser;

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

    #[traced_test]
    #[test]
    fn test_attribute_hash() {
        let a = Attribute::new(AttributeValue::Clip(true));
        let b = Attribute::new(AttributeValue::Clip(false));

        assert_ne!(a.xxhash(), b.xxhash());

        let a = Attribute::new(AttributeValue::Clip(true));
        let b = Attribute::new(AttributeValue::Clip(true));

        assert_eq!(a.xxhash(), b.xxhash());
    }

    #[traced_test]
    #[test]
    fn test_attributes_hash() {
        // Repeat these tests a number of times to check if the iterators are producing
        // attributes in a deterministic way
        for _ in 0..100 {
            // Should be equal
            let a = AttributeParser::parse_attributes("width:1, height:2").unwrap();
            let b = AttributeParser::parse_attributes("width:1, height:2").unwrap();
            assert_eq!(a.xxhash(), b.xxhash());

            // Flipping the order of attributes should also have equal hashes,
            // as long as the values stay the same.
            let a = AttributeParser::parse_attributes("height:2, width:1").unwrap();
            let b = AttributeParser::parse_attributes("width:1, height:2").unwrap();
            assert_eq!(a.xxhash(), b.xxhash());

            // Should not be equal
            let a = AttributeParser::parse_attributes("width:1, height:1").unwrap();
            let b = AttributeParser::parse_attributes("width:1, height:2").unwrap();
            assert_ne!(a.xxhash(), b.xxhash());
        }
    }
}
