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

use super::{comments, fields, generics, ids, punctuate, ws, Input, ParserExt, Result};
use crate::{
    highlight,
    punctuated::Punctuated,
    token::{self, Delimiter, Punctuation},
    Attributes, Comment, Enum, Name, Variant,
};

/// Encountered an invalid `enum` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse enum declaration"),
    code(mabo::parse::enum_def),
    help(
        "Expected enum declaration in the form `{}`",
        highlight::sample("enum <Name> {...}"),
    )
)]
#[rename(ParseEnumError)]
pub struct ParseError {
    /// Source location of the whole enum.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `enum` declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseEnumCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind, usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid enum name"),
        code(mabo::parse::enum_def::invalid_name),
        help(
            "Enum names must start with an uppercase letter ({}), followed by zero or more \
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
    /// Defined variant name is not considered valid.
    #[err(
        msg("Invalid variant name"),
        code(mabo::parse::enum_def::invalid_name),
        help(
            "Variant names must start with an uppercase letter ({}), followed by zero or more \
             alphanumeric characters ({})",
            highlight::value("A-Z"),
            highlight::value("A-Z, a-z, 0-9"),
        )
    )]
    InvalidVariantName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Invalid generic definition of the enum.
    #[forward]
    Generics(generics::ParseError),
    /// Invalid field in a named variant.
    #[forward]
    Field(fields::ParseError),
    /// Failed to parse the comments of a variant.
    #[forward]
    Comment(comments::ParseError),
    /// Invalid variant identifier.
    #[forward]
    Id(ids::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Enum<'i>, ParseError> {
    (
        terminated(token::Enum::NAME.span(), space1),
        cut_err((
            parse_name,
            opt(generics::parse.map_err(Cause::from)),
            preceded(space0, parse_variants),
        )),
    )
        .parse_next(input)
        .map(|(keyword, (name, generics, (brace, variants)))| Enum {
            comment: Comment::default(),
            attributes: Attributes::default(),
            keyword: keyword.into(),
            name,
            generics,
            brace,
            variants,
        })
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: input.location()..input.location(),
                cause,
            })
        })
}

pub(super) fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
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

fn parse_variants<'i>(
    input: &mut Input<'i>,
) -> Result<(token::Brace, Punctuated<Variant<'i>>), Cause> {
    (
        token::Brace::OPEN.span(),
        cut_err((
            punctuate(
                (parse_variant, ws(token::Comma::VALUE.span())),
                (parse_variant, opt(ws(token::Comma::VALUE.span()))),
            ),
            ws(token::Brace::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(brace_open, (variants, brace_close))| ((brace_open, brace_close).into(), variants))
}

fn parse_variant<'i>(input: &mut Input<'i>) -> Result<Variant<'i>, Cause> {
    (
        ws(comments::parse.map_err(Cause::from)),
        (
            preceded(space0, parse_variant_name.with_span()),
            preceded(space0, fields::parse.map_err(Cause::from)),
            opt(preceded(space0, ids::parse.map_err(Cause::from))),
        )
            .with_span(),
    )
        .parse_next(input)
        .map(|(comment, ((name, fields, id), span))| Variant {
            comment,
            name: name.into(),
            fields,
            id,
            span: span.into(),
        })
}

fn parse_variant_name<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    (one_of('A'..='Z'), alphanumeric0)
        .recognize()
        .parse_next(input)
        .map_err(|e| {
            e.map(|()| Cause::InvalidVariantName {
                at: input.location(),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_enum() {
        let err = ParseError {
            at: (0..31),
            cause: Cause::InvalidName { at: 5 },
        };

        println!(
            "{:?}",
            miette::Report::from(err).with_source_code("enum sample {\n    Variant @1,\n}")
        );
    }
}
