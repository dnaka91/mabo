use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser, ascii::alphanumeric0, combinator::opt, error::ErrMode, stream::Location, token::one_of,
};

use super::{Input, Result, punctuate, surround, ws};
use crate::{Generics, Name, highlight, token};

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
    Parser(usize),
    /// Defined name is not considered valid.
    #[err(msg("TODO!"), code(mabo::parse::generics::invalid_name), help("TODO!"))]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Generics<'i>, ParseError> {
    surround(punctuate(
        (ws(parse_name), ws(token::Comma::parser())),
        (ws(parse_name), opt(ws(token::Comma::parser()))),
    ))
    .parse_next(input)
    .map(|(angle, types)| Generics { angle, types })
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: input.current_token_start()..input.current_token_start(),
            cause,
        })
    })
}

fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (one_of('A'..='Z'), alphanumeric0)
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
