use std::collections::HashMap;

use colored::Colorize;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use tracing::{debug, warn};

use crate::{module::handle::ModuleHandle, parser::value::ValueParser};

use super::{error::ParseError, ParserContext, Value};

#[derive(Default, Debug, Clone)]
pub struct ModuleArguments {
    arguments: HashMap<String, Value>,
}

impl ModuleArguments {
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

    pub fn insert(&mut self, arg: ModuleArgument) {
        let name = arg.name.clone();
        if let Some(old) = self.arguments.insert(arg.name, arg.value) {
            warn!("Argument '{}' replaced old value: {}", name, old);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.arguments.get(name)
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

impl std::fmt::Display for ModuleArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Module {
    /// Parser context from the parent parser
    context: Option<ParserContext>,
    name: String,
    //args: Vec<ModuleArgument>,
    args: ModuleArguments,
    handle: Option<ModuleHandle<crate::module::test::FooEvent>>,
}

impl Module {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn args(&self) -> &ModuleArguments {
        &self.args
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.name.bright_magenta())?;

        write!(f, "{}", self.args);

        write!(f, " handle={:?}]", self.handle)?;

        Ok(())
    }
}

impl std::hash::Hash for Module {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.args.hash(state);
    }
}

#[derive(Parser)]
#[grammar = "parser/module.pest"]
pub struct ModuleParser;

impl ModuleParser {
    pub fn parse_str(data: &str, context: ParserContext) -> Result<Module, ParseError> {
        debug!("Parsing module {data}");
        let pairs = ModuleParser::parse(Rule::module, data)?;

        let mut module = Module::default();
        module.context = Some(context);

        if let Some(root) = pairs.into_iter().last() {
            for pair in root.into_inner() {
                match pair.as_rule() {
                    Rule::module_name => {
                        module.name = pair.as_str().into();
                    }
                    Rule::module_arguments => {
                        Self::parse_arguments(
                            pair,
                            &mut module.args,
                            module.context.as_ref().unwrap(),
                        )?;
                    }

                    // Return the module when the EOI rule is emitted
                    Rule::EOI => return Ok(module),

                    // Handle unsupported rules
                    _ => {
                        return Err(ParseError::UnsupportedRule(format!(
                            "{}: {} {:?}",
                            file!(),
                            line!(),
                            pair.as_rule()
                        )))
                    }
                }
            }
        } else {
            return Err(ParseError::Missing("root pair"));
        }

        Err(ParseError::Missing("EOI not emitted"))
    }

    fn parse_argument(
        pair: Pair<Rule>,
        context: &ParserContext,
    ) -> Result<ModuleArgument, ParseError> {
        let mut arg = ModuleArgument::default();

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::argument_name => {
                    arg.name = pair.as_str().into();
                }
                Rule::value => {
                    arg.value = ValueParser::parse_str(pair.as_str(), context)?;
                }
                // Handle unsupported rules
                _ => {
                    return Err(ParseError::UnsupportedRule(format!(
                        "{}: {} {:?}",
                        file!(),
                        line!(),
                        pair.as_rule()
                    )));
                }
            }
        }

        Ok(arg)
    }

    fn parse_arguments(
        pair: Pair<Rule>,
        dest: &mut ModuleArguments,
        context: &ParserContext,
    ) -> Result<(), ParseError> {
        for pair in pair.into_inner() {
            //println!("ARGUMENT PAIR {pair:?}")
            match pair.as_rule() {
                Rule::argument => {
                    let argument = Self::parse_argument(pair, context)?;
                    dest.insert(argument);
                }
                // Handle unsupported rules
                _ => {
                    return Err(ParseError::UnsupportedRule(format!(
                        "{}: {} {:?}",
                        file!(),
                        line!(),
                        pair.as_rule()
                    )))
                }
            }
        }

        Ok(())
    }
}
