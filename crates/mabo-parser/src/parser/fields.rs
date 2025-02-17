use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::space0,
    combinator::{opt, peek, preceded},
    dispatch,
    error::ErrMode,
    stream::{Location, Stream},
    token::{any, one_of, take_while},
};

use super::{Input, ParserExt, Result, comments, ids, punctuate, surround, types, ws};
use crate::{
    Fields, Name, NamedField, UnnamedField, highlight, location, punctuated::Punctuated, token,
};

/// Encountered an invalid field declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse fields declaration"),
    code(mabo::parse::id),
    help(
        "Expected fields declaration in the form `{}`, `{}` or `{}`",
        highlight::sample("{ <named>, <named>, ... }"),
        highlight::sample("( <unnamed>, <unnamed>, ... )"),
        highlight::sample("_nothing_"),
    )
)]
#[rename(ParseFieldsError)]
pub struct ParseError {
    /// Source location of the whole id.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a fields declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseFieldsCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(usize),
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid field name"),
        code(mabo::parse::fields::named::invalid_name),
        help(
            "Field names must start with a lowercase letter ({}), followed by zero or more \
             lowercase alphanumeric characters or underscores ({})",
            highlight::value("a-z"),
            highlight::value("a-z, 0-9, _"),
        )
    )]
    InvalidName {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Invalid type declaration.
    #[forward]
    Type(types::ParseError),
    /// Invalid field identifier.
    #[forward]
    Id(ids::ParseError),
    /// Failed parsing field comments.
    #[forward]
    Comment(comments::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Fields<'i>, ParseError> {
    let start = input.checkpoint();

    dispatch!(
        peek(any);
        '{' => parse_named.map(|(brace, fields)| Fields::Named(brace, fields)),
        '(' => parse_unnamed.map(|(paren, fields)| Fields::Unnamed(paren, fields)),
        _ => parse_unit.map(|()| Fields::Unit),
    )
    .parse_next(input)
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: location::from_until(*input, &start, [',', '\n']),
            cause,
        })
    })
}

fn parse_named<'i>(
    input: &mut Input<'i>,
) -> Result<(token::Brace, Punctuated<NamedField<'i>>), Cause> {
    surround(punctuate(
        (parse_named_field, ws(token::Comma::parser())),
        (parse_named_field, opt(ws(token::Comma::parser()))),
    ))
    .parse_next(input)
}

fn parse_unnamed<'i>(
    input: &mut Input<'i>,
) -> Result<(token::Parenthesis, Punctuated<UnnamedField<'i>>), Cause> {
    surround(punctuate(
        (parse_unnamed_field, ws(token::Comma::parser())),
        (parse_unnamed_field, opt(ws(token::Comma::parser()))),
    ))
    .parse_next(input)
}

fn parse_unit(input: &mut Input<'_>) -> Result<(), Cause> {
    ().parse_next(input)
}

fn parse_unnamed_field<'i>(input: &mut Input<'i>) -> Result<UnnamedField<'i>, Cause> {
    (
        ws(types::parse.map_err(Cause::from)),
        opt(preceded(space0, ids::parse.map_err(Cause::from))),
    )
        .with_span()
        .parse_next(input)
        .map(|((ty, id), span)| UnnamedField {
            ty,
            id,
            span: span.into(),
        })
}

fn parse_named_field<'i>(input: &mut Input<'i>) -> Result<NamedField<'i>, Cause> {
    (
        ws(comments::parse.map_err(Cause::from)),
        (
            preceded(space0, parse_field_name),
            preceded(space0, token::Colon::parser()),
            preceded(space0, types::parse.map_err(Cause::from)),
            opt(preceded(space0, ids::parse.map_err(Cause::from))),
        )
            .with_span(),
    )
        .parse_next(input)
        .map(|(comment, ((name, colon, ty, id), span))| NamedField {
            comment,
            name,
            colon,
            ty,
            id,
            span: span.into(),
        })
}

fn parse_field_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
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
