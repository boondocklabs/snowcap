use colored::Colorize;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use tracing::debug;

use crate::{module::handle::ModuleHandle, parser::value::ValueParser};

use super::{error::ParseError, ParserContext, Value};

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
    args: Vec<ModuleArgument>,
    handle: Option<ModuleHandle<crate::module::test::FooEvent>>,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.name.bright_magenta(),)?;

        if !self.args.is_empty() {
            write!(f, " args=")?;
            let mut iter = self.args.iter().peekable();

            while let Some(arg) = iter.next() {
                write!(f, "{}", arg)?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
        }

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
        dest: &mut Vec<ModuleArgument>,
        context: &ParserContext,
    ) -> Result<(), ParseError> {
        for pair in pair.into_inner() {
            //println!("ARGUMENT PAIR {pair:?}")
            match pair.as_rule() {
                Rule::argument => {
                    let argument = Self::parse_argument(pair, context)?;
                    dest.push(argument);
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
