use std::ops::Range;

use stef_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::space1,
    combinator::{alt, cut_err, opt, preceded, separated, terminated},
    error::ErrorKind,
    stream::{Location, Stream},
    token::{one_of, take_while},
    Parser,
};

use super::{enums, structs, Input, ParserExt, Result};
use crate::{highlight, location, Import, Name};

/// Encountered an invalid `use` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse use declaration"),
    code(stef::parse::use_def),
    help(
        "Expected import declaration in the form `{}`",
        highlight::sample("use <path...>::<element>;"),
    )
)]
#[rename(ParseImportError)]
pub struct ParseError {
    /// Source location of the whole use statement.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `use` declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseImportCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind, usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid segment name"),
        code(stef::parse::import::segment::invalid_name),
        help(
            "Import path names must start with a lowercase letter ({}), followed by zero or more \
             lowercase alphanumeric characters or underscores ({})",
            highlight::value("a-z"),
            highlight::value("a-z, 0-9, _"),
        )
    )]
    InvalidSegmentName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Invalid struct name.
    #[forward]
    StructName(structs::Cause),
    /// Invalid enum name.
    #[forward]
    EnumName(enums::Cause),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Import<'i>, ParseError> {
    let start = input.checkpoint();

    preceded(
        ("use", space1),
        cut_err(terminated(
            (
                separated(1.., parse_segment, "::"),
                opt(preceded(
                    "::",
                    alt((
                        structs::parse_name.map_err(Cause::from),
                        enums::parse_name.map_err(Cause::from),
                    )),
                )),
            ),
            ';',
        )),
    )
    .parse_next(input)
    .map(|(segments, element)| Import { segments, element })
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: location::from_until(*input, start, [';']),
            cause,
        })
    })
}

pub(super) fn parse_segment<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
        .recognize()
        .with_span()
        .parse_next(input)
        .map(Into::into)
        .map_err(|e| {
            e.map(|()| Cause::InvalidSegmentName {
                at: input.location(),
            })
        })
}
