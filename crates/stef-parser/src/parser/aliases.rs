use std::ops::Range;

use stef_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::{space0, space1},
    combinator::{cut_err, delimited, preceded},
    error::ErrorKind,
    stream::Location,
    Parser,
};

use super::{types, Input, ParserExt, Result};
use crate::{highlight, Comment, TypeAlias};

/// Encountered an invalid `type` alias declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse type alias declaration"),
    code(stef::parse::alias_def),
    help(
        "Expected type alias declaration in the form `{}`",
        highlight::sample("type <Alias> = <Type>;"),
    )
)]
#[rename(ParseAliasError)]
pub struct ParseError {
    /// Source location of the whole id.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `type` alias declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseAliasCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind),
    /// Invalid type declaration.
    #[forward]
    Type(types::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<TypeAlias<'i>, ParseError> {
    preceded(
        ("type", space1),
        cut_err((
            types::parse.map_err(Cause::from),
            delimited(
                (space0, '='),
                preceded(space0, types::parse.map_err(Cause::from)),
                (space0, ';'),
            ),
        )),
    )
    .parse_next(input)
    .map(|(alias, target)| TypeAlias {
        comment: Comment::default(),
        alias,
        target,
    })
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: input.location()..input.location(),
            cause,
        })
    })
}
