use std::{
    cell::{Ref, RefCell},
    sync::Arc,
};

use crate::Value;

#[derive(Debug, Default)]
pub struct Attributes(Arc<RefCell<Vec<Attribute>>>);

impl Attributes {
    pub fn get(&self, name: &str) -> Option<Attribute> {
        for attr in &*self.0.borrow() {
            if attr.name.as_str() == name {
                return Some(attr.clone());
            }
        }
        None
    }

    pub fn set(&self, name: &str, value: Value) {
        for attr in &*self.0.borrow() {
            if attr.name.as_str() == name {
                *attr.value.borrow_mut() = value;
                return;
            }
        }

        self.0.borrow_mut().push(Attribute {
            name: name.to_string(),
            value: Arc::new(RefCell::new(value)),
        })
    }

    // Function to return an iterator over the inner Vec<Attribute>
    pub fn iter(&self) -> impl Iterator<Item = Attribute> + '_ {
        // Borrow the inner Vec immutably and return its iterator
        self.0.borrow().clone().into_iter()
    }
}

impl IntoIterator for Attributes {
    type Item = Attribute;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow().clone().into_iter()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = Attribute;

    type IntoIter = std::vec::IntoIter<Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow().clone().into_iter()
    }
}

impl FromIterator<Attribute> for Attributes {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        Attributes(Arc::new(RefCell::new(c)))
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    value: Arc<RefCell<Value>>,
}

impl Attribute {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn value<'a>(&'a self) -> Ref<'a, Value> {
        (*self.value).borrow()
    }

    pub fn new(name: String, value: Value) -> Self {
        Self {
            name,
            value: Arc::new(RefCell::new(value)),
        }
    }
}
