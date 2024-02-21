use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::alphanumeric0,
    combinator::{cut_err, opt},
    error::ErrorKind,
    stream::Location,
    token::one_of,
    Parser,
};

use super::{punctuate, ws, Input, Result};
use crate::{
    highlight,
    token::{self, Delimiter, Punctuation},
    Generics, Name,
};

/// Encountered an invalid `<...>` generics declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse generics declaration"),
    code(mabo::parse::generics),
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
    Parser(ErrorKind, usize),
    /// Defined name is not considered valid.
    #[err(msg("TODO!"), code(mabo::parse::generics::invalid_name), help("TODO!"))]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Generics<'i>, ParseError> {
    (
        token::Angle::OPEN.span(),
        cut_err((
            punctuate(
                (ws(parse_name), ws(token::Comma::VALUE.span())),
                (ws(parse_name), opt(ws(token::Comma::VALUE.span()))),
            ),
            ws(token::Angle::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(angle_open, (types, angle_close))| Generics {
            angle: (angle_open, angle_close).into(),
            types,
        })
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
