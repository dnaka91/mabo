use std::ops::Range;

use stef_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::alphanumeric0,
    combinator::{cut_err, preceded, separated, terminated},
    error::ErrorKind,
    stream::Location,
    token::one_of,
    Parser,
};

use super::{ws, Input, Result};
use crate::{highlight, Generics, Name};

/// Encountered an invalid `<...>` generics declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse generics declaration"),
    code(stef::parse::generics),
    help(
        "Expected generics declaration in the form `{}`",
        highlight::sample("<T1, T2, ...>"),
    )
)]
#[rename(ParseGenericsError)]
pub struct ParseError {
    /// Source location of the whole comment.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `<...>` generics declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseGenericsCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind),
    /// Defined name is not considered valid.
    #[err(msg("TODO!"), code(stef::parse::generics::invalid_name), help("TODO!"))]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Generics<'i>, ParseError> {
    preceded(
        '<',
        cut_err(terminated(separated(1.., ws(parse_name), ws(',')), ws('>'))),
    )
    .parse_next(input)
    .map(Generics)
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: input.location()..input.location(),
            cause,
        })
    })
}

fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (one_of('A'..='Z'), alphanumeric0)
        .recognize()
        .with_span()
        .parse_next(input)
        .map(Into::into)
        .map_err(|e| {
            e.map(|()| Cause::InvalidName {
                at: input.location(),
            })
        })
}
