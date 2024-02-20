use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::space0,
    combinator::{cut_err, opt, peek, preceded, repeat},
    dispatch,
    error::ErrorKind,
    stream::{Location, Stream},
    token::{any, one_of, take_while},
    Parser,
};

use super::{comments, ids, types, ws, Input, ParserExt, Result};
use crate::{
    highlight, location,
    token::{self, Delimiter, Punctuation},
    Fields, Name, NamedField, UnnamedField,
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
    Parser(ErrorKind, usize),
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

fn parse_named<'i>(input: &mut Input<'i>) -> Result<(token::Brace, Vec<NamedField<'i>>), Cause> {
    (
        token::Brace::OPEN.span(),
        cut_err((
            repeat(1.., parse_named_field),
            ws(token::Brace::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(brace_open, (fields, brace_close))| ((brace_open, brace_close).into(), fields))
}

fn parse_unnamed<'i>(
    input: &mut Input<'i>,
) -> Result<(token::Parenthesis, Vec<UnnamedField<'i>>), Cause> {
    (
        token::Parenthesis::OPEN.span(),
        cut_err((
            repeat(1.., parse_unnamed_field),
            ws(token::Parenthesis::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(paren_open, (fields, paren_close))| ((paren_open, paren_close).into(), fields))
}

fn parse_unit(input: &mut Input<'_>) -> Result<(), Cause> {
    ().parse_next(input)
}

fn parse_unnamed_field<'i>(input: &mut Input<'i>) -> Result<UnnamedField<'i>, Cause> {
    (
        ws(types::parse.map_err(Cause::from)),
        opt(preceded(space0, ids::parse.map_err(Cause::from))),
        opt(ws(token::Comma::VALUE.span())),
    )
        .with_span()
        .parse_next(input)
        .map(|((ty, id, comma), span)| UnnamedField {
            ty,
            id,
            comma: comma.map(Into::into),
            span: span.into(),
        })
}

fn parse_named_field<'i>(input: &mut Input<'i>) -> Result<NamedField<'i>, Cause> {
    (
        ws(comments::parse.map_err(Cause::from)),
        (
            preceded(space0, parse_field_name),
            preceded(space0, token::Colon::VALUE.span()),
            preceded(space0, types::parse.map_err(Cause::from)),
            opt(preceded(space0, ids::parse.map_err(Cause::from))),
            opt(ws(token::Comma::VALUE.span())),
        )
            .with_span(),
    )
        .parse_next(input)
        .map(
            |(comment, ((name, colon, ty, id, comma), span))| NamedField {
                comment,
                name,
                colon: colon.into(),
                ty,
                id,
                comma: comma.map(Into::into),
                span: span.into(),
            },
        )
}

fn parse_field_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
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
