use colored::Colorize;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use tracing::debug;

use crate::{
    module::{
        argument::{ModuleArgument, ModuleArguments},
        ModuleHandleId,
    },
    parser::value::ValueParser,
};

use super::{error::ParseError, ParserContext};

/// Parsed Module from the grammar and tree node representation
#[derive(Default, Debug, Clone)]
pub struct Module {
    /// Parser context from the parent parser
    context: Option<ParserContext>,

    /// Module name for locating the module in the global [`ModuleRegistry`]
    name: String,

    /// Module Arguments passed to module instantiation
    args: ModuleArguments,

    /// Module Handle ID set after module is instantiated
    handle_id: Option<ModuleHandleId>,
}

impl Module {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn args(&self) -> &ModuleArguments {
        &self.args
    }

    pub fn handle_id(&self) -> Option<ModuleHandleId> {
        self.handle_id
    }

    pub fn set_handle_id(&mut self, handle_id: ModuleHandleId) {
        debug!("Module {} set handle {}", self, handle_id);
        self.handle_id = Some(handle_id);
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.name.bright_magenta())?;

        if self.args.len() > 0 {
            write!(f, " {}", self.args)?;
        }

        if let Some(handle_id) = self.handle_id() {
            write!(f, " handle_id: {handle_id}")?;
        }

        write!(f, "]")?;

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
                    arg.set_name(pair.as_str().into());
                }
                Rule::value => {
                    arg.set_value(ValueParser::parse_str(pair.as_str(), context)?);
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
