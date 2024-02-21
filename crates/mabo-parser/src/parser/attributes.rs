use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::space0,
    combinator::{alt, cut_err, opt, preceded, repeat, separated, terminated},
    error::ErrorKind,
    stream::Location,
    token::{one_of, take_while},
    Parser,
};

use super::{literals, ws, Input, ParserExt, Result};
use crate::{
    highlight,
    token::{self, Delimiter},
    Attribute, AttributeValue, Attributes, Literal,
};

/// Encountered an invalid `#[...]` attribute declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse struct declaration"),
    code(mabo::parse::struct_def),
    help(
        "Expected struct declaration in the form `{}`",
        highlight::sample("struct <Name> {...}"),
    )
)]
#[rename(ParseAttributeError)]
pub struct ParseError {
    /// Source location of the whole attribute.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `#[...]` attribute declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseAttributeCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind, usize),
    /// Invalid literal for the attribute value.
    #[forward]
    Literal(literals::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Attributes<'i>, ParseError> {
    let start = input.location();

    repeat(0.., terminated(parse_attribute, '\n'))
        .fold(Vec::new, |mut acc, attrs| {
            acc.extend(attrs);
            acc
        })
        .parse_next(input)
        .map(Attributes)
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: start..start,
                cause,
            })
        })
}

fn parse_attribute<'i>(input: &mut Input<'i>) -> Result<Vec<Attribute<'i>>, Cause> {
    preceded(
        (token::Pound::parser(), token::Bracket::OPEN),
        cut_err(terminated(
            terminated(
                separated(
                    1..,
                    ws((parse_name, parse_value)).map(|(name, value)| Attribute { name, value }),
                    ws(token::Comma::parser()),
                ),
                opt(token::Comma::parser()),
            ),
            ws(token::Bracket::CLOSE),
        )),
    )
    .parse_next(input)
}

fn parse_name<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    (
        one_of('a'..='z'),
        take_while(0.., ('a'..='z', '0'..='9', '_')),
    )
        .recognize()
        .parse_next(input)
}

fn parse_value<'i>(input: &mut Input<'i>) -> Result<AttributeValue<'i>, Cause> {
    alt((
        parse_multi_value.map(AttributeValue::Multi),
        parse_single_value.map(AttributeValue::Single),
        parse_unit_value.map(|()| AttributeValue::Unit),
    ))
    .parse_next(input)
}

fn parse_multi_value<'i>(input: &mut Input<'i>) -> Result<Vec<Attribute<'i>>, Cause> {
    preceded(
        token::Parenthesis::OPEN,
        cut_err(terminated(
            terminated(
                separated(
                    1..,
                    ws((parse_name, parse_value)).map(|(name, value)| Attribute { name, value }),
                    ws(token::Comma::parser()),
                ),
                opt(token::Comma::parser()),
            ),
            ws(token::Parenthesis::CLOSE),
        )),
    )
    .parse_next(input)
}

fn parse_single_value(input: &mut Input<'_>) -> Result<Literal, Cause> {
    preceded(
        (space0, token::Equal::parser(), space0),
        literals::parse.map_err(Cause::from),
    )
    .parse_next(input)
}

fn parse_unit_value(input: &mut Input<'_>) -> Result<(), Cause> {
    ().parse_next(input)
}
