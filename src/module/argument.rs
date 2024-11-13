use std::collections::HashMap;

use tracing::warn;

use crate::{
    parser::{value::ValueParser, ParserContext},
    Value,
};

use super::error::ModuleError;

/// A set of arguments for Modules parsed from the grammar
#[derive(Default, Debug, Clone)]
pub struct ModuleArguments {
    arguments: HashMap<String, Value>,
}

impl ModuleArguments {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder pattern to add arguments using the parser
    pub fn arg(mut self, arg: &str, value: &str) -> Self {
        let value = ValueParser::parse_str(value, &ParserContext::default()).unwrap();

        let arg = ModuleArgument::new(arg.to_string(), value);
        self.insert(arg);

        self
    }

    /// Sort the set of [`ModuleArgument`] items in a determinstic way.
    /// Returns a Vec of the sorted [`ModuleArgument`] items.
    pub fn sort(&self) -> Vec<ModuleArgument> {
        let mut v: Vec<ModuleArgument> = self
            .arguments
            .iter()
            .map(|(k, v)| ModuleArgument {
                name: k.clone(),
                value: v.clone(),
            })
            .collect();

        v.sort_by(|this, other| this.name.cmp(&other.name));
        v
    }

    /// Insert a new [`ModuleArgument`]
    pub fn insert(&mut self, arg: ModuleArgument) {
        let name = arg.name.clone();
        if let Some(old) = self.arguments.insert(arg.name, arg.value) {
            warn!("Argument '{}' replaced old value: {}", name, old);
        }
    }

    /// Get a reference to the [`Value`] of a [`ModuleArgument`] specified by the supplied name
    pub fn get(&self, name: &str) -> Result<&Value, ModuleError> {
        self.arguments
            .get(name)
            .ok_or(ModuleError::MissingArgument(name.to_string()))
    }

    /// Get the number of Arguments
    pub fn len(&self) -> usize {
        self.arguments.len()
    }
}

impl std::fmt::Display for ModuleArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.sort();

        if !v.is_empty() {
            write!(f, "args=")?;
            let mut iter = v.iter().peekable();

            while let Some(arg) = iter.next() {
                write!(f, "{}", arg)?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
        }

        Ok(())
    }
}

impl std::hash::Hash for ModuleArguments {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Collect the arguments into a vec, and sort them by name
        // for deterministic argument hashing.

        for arg in &self.sort() {
            arg.hash(state)
        }
    }
}

#[derive(Default, Debug, Hash, Clone)]
pub struct ModuleArgument {
    name: String,
    value: Value,
}

impl ModuleArgument {
    pub fn new(name: String, value: Value) -> Self {
        Self { name, value }
    }

    /// Set the argument name
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    /// Set the argument value
    pub fn set_value(&mut self, value: Value) {
        self.value = value
    }

    /// Get the argument name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the argument value
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl std::fmt::Display for ModuleArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}
