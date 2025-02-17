use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::space1,
    combinator::{alt, cut_err, opt, separated, terminated},
    error::ErrMode,
    stream::{Location, Stream},
    token::{one_of, take_while},
};

use super::{Input, ParserExt, Result, enums, structs};
use crate::{Import, Name, highlight, location, token};

/// Encountered an invalid `use` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse use declaration"),
    code(mabo::parse::use_def),
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
    Parser(usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid segment name"),
        code(mabo::parse::import::segment::invalid_name),
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

    (
        terminated(token::Use::parser(), space1),
        cut_err((
            (
                separated(1.., parse_segment, token::DoubleColon::parser()),
                opt((
                    token::DoubleColon::parser(),
                    alt((
                        structs::parse_name.map_err(Cause::from),
                        enums::parse_name.map_err(Cause::from),
                    )),
                )),
            )
                .with_taken()
                .with_span(),
            token::Semicolon::parser(),
        )),
    )
        .parse_next(input)
        .map(
            |(keyword, ((((segments, element), full), range), semicolon))| Import {
                keyword,
                full: (full, range).into(),
                segments,
                element,
                semicolon,
            },
        )
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: location::from_until(*input, &start, [';']),
                cause,
            })
        })
}

pub(super) fn parse_segment<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
        .take()
        .with_span()
        .parse_next(input)
        .map(Into::into)
        .map_err(|e: ErrMode<_>| {
            e.map(|()| Cause::InvalidSegmentName {
                at: input.current_token_start(),
            })
        })
}
