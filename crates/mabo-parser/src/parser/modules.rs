use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::{space0, space1},
    combinator::{cut_err, preceded, repeat, terminated},
    error::ErrMode,
    stream::{Location, Stream},
    token::{one_of, take_while},
};

use super::{Input, ParserExt, Result, parse_definition, surround, ws};
use crate::{Comment, Module, Name, error::ParseDefinitionError, highlight, location, token};

/// Encountered an invalid `mod` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse id declaration"),
    code(mabo::parse::mod_def),
    help(
        "Expected module declaration in the form `{}`",
        highlight::sample("mod <name> {...}"),
    )
)]
#[rename(ParseModuleError)]
pub struct ParseError {
    /// Source location of the whole module.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `mod` declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseModuleCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid module name"),
        code(mabo::parse::module::invalid_name),
        help(
            "Module names must start with a lowercase letter ({}), followed by zero or more \
             lowercase alphanumeric characters or underscores ({})",
            highlight::value("a-z"),
            highlight::value("a-z, 0-9, _"),
        )
    )]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Invalid definition of any element within the module.
    #[forward]
    Definition(Box<ParseDefinitionError>),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Module<'i>, ParseError> {
    let start = input.checkpoint();

    (
        terminated(token::Mod::parser(), space1),
        cut_err((
            parse_name,
            preceded(
                space0,
                surround(repeat(0.., ws(parse_definition.map_err2(Cause::from)))),
            ),
        )),
    )
        .parse_next(input)
        .map(|(keyword, (name, (brace, definitions)))| Module {
            comment: Comment::default(),
            keyword,
            name,
            brace,
            definitions,
        })
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: location::from_until(*input, &start, ['}', '\n']),
                cause,
            })
        })
}

fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
        .take()
        .with_span()
        .parse_next(input)
        .map(Into::into)
        .map_err(|e: ErrMode<_>| {
            e.map(|()| Cause::InvalidName {
                at: input.current_token_start(),
            })
        })
}
