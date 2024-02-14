use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::space0,
    combinator::{cut_err, delimited, opt, peek, preceded, separated, terminated},
    dispatch,
    error::ErrorKind,
    stream::{Location, Stream},
    token::{any, one_of, take_while},
    Parser,
};

use super::{comments, ids, types, ws, Input, ParserExt, Result};
use crate::{highlight, location, Fields, Name, NamedField, UnnamedField};

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
        '{' => parse_named.map(Fields::Named),
        '(' => parse_unnamed.map(Fields::Unnamed),
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

fn parse_named<'i>(input: &mut Input<'i>) -> Result<Vec<NamedField<'i>>, Cause> {
    preceded(
        '{',
        cut_err(terminated(
            terminated(separated(1.., parse_named_field, ws(',')), opt(ws(','))),
            ws('}'),
        )),
    )
    .parse_next(input)
}

fn parse_unnamed<'i>(input: &mut Input<'i>) -> Result<Vec<UnnamedField<'i>>, Cause> {
    preceded(
        '(',
        cut_err(terminated(
            terminated(separated(1.., parse_unnamed_field, ws(',')), opt(ws(','))),
            ws(')'),
        )),
    )
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
            delimited(space0, parse_field_name, ':'),
            preceded(space0, types::parse.map_err(Cause::from)),
            opt(preceded(space0, ids::parse.map_err(Cause::from))),
        )
            .with_span(),
    )
        .parse_next(input)
        .map(|(comment, ((name, ty, id), span))| NamedField {
            comment,
            name,
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
