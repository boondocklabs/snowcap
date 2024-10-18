use std::{ops::Deref, sync::Arc, time::Duration};

use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLock};
use strum::{EnumDiscriminants, EnumIter};

use crate::Error;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
#[strum_discriminants(name(AttributeType))]
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
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attribute_type: AttributeType = self.into();
        write!(f, "{:?}", attribute_type)
    }
}

#[derive(Default, Clone)]
pub struct Attributes(Arc<RwLock<Vec<Attribute>>>);

impl Attributes {
    pub fn push(&mut self, attr: Attribute) -> Result<(), crate::Error> {
        // NOTE: This will panic if the lock is already locked
        // TODO: Propagate error
        if let Some(mut guard) = self.0.try_write() {
            guard.push(attr);
            Ok(())
        } else {
            Err(Error::Deadlock(format!(
                "RwLock try_write() failed in Attributes::push() {:#?}",
                attr
            )))
        }
    }

    pub fn get(self, find: AttributeType) -> Option<Attribute> {
        for attr in self.0.read().iter() {
            let attr_type: AttributeType = (&**attr).into();

            if find == attr_type {
                return Some(attr.clone());
            }
        }

        None
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
        match self.0.try_read_arc_for(Duration::from_secs(1)) {
            Some(guard) => AttributeIter { guard, index: 0 },
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
    guard: ArcRwLockReadGuard<RawRwLock, Vec<Attribute>>,
    index: usize,
}

impl Iterator for AttributeIter {
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.guard.get(self.index).map(|item| item.clone());
        self.index += 1;
        res
    }
}

impl FromIterator<Attribute> for Attributes {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        let attributes: Vec<Attribute> = iter.into_iter().collect();
        Attributes(Arc::new(RwLock::new(attributes)))
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
    pub fn value<'a>(&'a self) -> &'a AttributeValue {
        &self.value
    }

    pub fn value_mut<'a>(&'a mut self) -> &'a mut AttributeValue {
        &mut self.value
    }

    pub fn new(value: AttributeValue) -> Self {
        Self { value }
    }
}
