use std::ops::Range;

use stef_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::{alphanumeric0, space0, space1},
    combinator::{cut_err, opt, preceded},
    error::ErrorKind,
    stream::{Location, Stream},
    token::one_of,
    Parser,
};

use super::{fields, generics, Input, ParserExt, Result};
use crate::{highlight, location, Attributes, Comment, Struct};

/// Encountered an invalid `struct` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse struct declaration"),
    code(stef::parse::struct_def),
    help(
        "Expected struct declaration in the form `{}`",
        highlight::sample("struct <Name> {...}"),
    )
)]
#[rename(ParseStructError)]
pub struct ParseError {
    /// Source location of the whole struct.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `struct` declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseStructCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid struct name"),
        code(stef::parse::struct_def::invalid_name),
        help(
            "Struct names must start with an uppercase letter ({}), followed by zero or more \
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
    /// Invalid sturct generics declaration.
    #[forward]
    Generics(super::generics::ParseError),
    /// Invalid declaration of struct fields.
    #[forward]
    Fields(fields::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Struct<'i>, ParseError> {
    let start = input.checkpoint();

    preceded(
        ("struct", space1),
        cut_err((
            parse_name,
            opt(generics::parse.map_err(Cause::Generics)).map(Option::unwrap_or_default),
            preceded(space0, fields::parse.map_err(Cause::Fields)),
        )),
    )
    .parse_next(input)
    .map(|(name, generics, kind)| Struct {
        comment: Comment::default(),
        attributes: Attributes::default(),
        name,
        generics,
        fields: kind,
    })
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: location::from_until(*input, start, ['}', '\n']),
            cause,
        })
    })
}

pub(super) fn parse_name<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    (one_of('A'..='Z'), alphanumeric0)
        .recognize()
        .parse_next(input)
        .map_err(|e| {
            e.map(|()| Cause::InvalidName {
                at: input.location(),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_struct() {
        let err = ParseError {
            at: (0..36),
            cause: Cause::InvalidName { at: 7 },
        };

        println!(
            "{:?}",
            miette::Report::from(err).with_source_code("struct sample {\n    value: u32 @1,\n}")
        );
    }
}
