use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::{alphanumeric0, space0, space1},
    combinator::{cut_err, opt, preceded, terminated},
    error::ErrMode,
    stream::Location,
    token::one_of,
};

use super::{Input, ParserExt, Result, comments, fields, generics, ids, punctuate, surround, ws};
use crate::{Attributes, Comment, Enum, Name, Variant, highlight, punctuated::Punctuated, token};

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
    Parser(usize),
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
        terminated(token::Enum::parser(), space1),
        cut_err((
            parse_name,
            opt(generics::parse.map_err2(Cause::from)),
            preceded(space0, parse_variants),
        )),
    )
        .parse_next(input)
        .map(|(keyword, (name, generics, (brace, variants)))| Enum {
            comment: Comment::default(),
            attributes: Attributes::default(),
            keyword,
            name,
            generics,
            brace,
            variants,
        })
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: input.current_token_start()..input.current_token_start(),
                cause,
            })
        })
}

pub(super) fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
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

fn parse_variants<'i>(
    input: &mut Input<'i>,
) -> Result<(token::Brace, Punctuated<Variant<'i>>), Cause> {
    surround(punctuate(
        (parse_variant, ws(token::Comma::parser())),
        (parse_variant, opt(ws(token::Comma::parser()))),
    ))
    .parse_next(input)
}

fn parse_variant<'i>(input: &mut Input<'i>) -> Result<Variant<'i>, Cause> {
    (
        ws(comments::parse.map_err2(Cause::from)),
        (
            preceded(space0, parse_variant_name.with_span()),
            preceded(space0, fields::parse.map_err2(Cause::from)),
            opt(preceded(space0, ids::parse.map_err2(Cause::from))),
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
        .take()
        .parse_next(input)
        .map_err(|e: ErrMode<_>| {
            e.map(|()| Cause::InvalidVariantName {
                at: input.current_token_start(),
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
