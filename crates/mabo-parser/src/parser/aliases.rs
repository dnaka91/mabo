use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::{alphanumeric0, space0, space1},
    combinator::{cut_err, opt, preceded, terminated},
    error::ErrorKind,
    stream::Location,
    token::one_of,
    Parser,
};

use super::{generics, types, Input, ParserExt, Result};
use crate::{highlight, token, Comment, Name, TypeAlias};

/// Encountered an invalid `type` alias declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse type alias declaration"),
    code(mabo::parse::alias_def),
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
    Parser(ErrorKind, usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid alias name"),
        code(mabo::parse::alias_def::invalid_name),
        help(
            "Alias names must start with an uppercase letter ({}), followed by zero or more \
             alphanumeric characters ({})",
            highlight::value("A-Z"),
            highlight::value("A-Z, a-z, 0-9"),
        )
    )]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Invalid alias generics declaration.
    #[forward]
    Generics(generics::ParseError),
    /// Invalid type declaration.
    #[forward]
    Type(types::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<TypeAlias<'i>, ParseError> {
    (
        terminated(token::Type::parser(), space1),
        cut_err((
            parse_name,
            opt(generics::parse.map_err(Cause::Generics)),
            preceded(space0, token::Equal::parser()),
            preceded(space0, types::parse.map_err(Cause::from)),
            preceded(space0, token::Semicolon::parser()),
        )),
    )
        .parse_next(input)
        .map(
            |(keyword, (name, generics, equal, target, semicolon))| TypeAlias {
                comment: Comment::default(),
                keyword,
                name,
                generics,
                equal,
                target,
                semicolon,
            },
        )
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
