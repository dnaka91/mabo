use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::{dec_uint, space0},
    combinator::{alt, cut_err, empty, fail, opt, preceded, separated, terminated},
    dispatch,
    error::ErrorKind,
    stream::Location,
    token::{literal, one_of, take_while},
    Parser,
};

use super::{imports, ws, Input, ParserExt, Result};
use crate::{
    highlight,
    token::{self, Delimiter, Punctuation},
    DataType, ExternalType, Name, Type,
};

/// Encountered an invalid type definition.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse type definition"),
    code(mabo::parse::type_def),
    help(
        "Expected type definition in the form `{}`",
        highlight::sample("<Name>"),
    )
)]
#[rename(ParseTypeError)]
pub struct ParseError {
    /// Source location of the whole type definition.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a type definition was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseTypeCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(ErrorKind, usize),
    /// Invalid type declaration.
    #[forward]
    Type(Box<ParseError>),
    /// Invalid path segment.
    #[forward]
    Segment(Box<imports::Cause>),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Type<'i>, ParseError> {
    let start = input.location();

    alt((
        parse_basic,
        parse_generic,
        parse_tuple,
        parse_array,
        parse_external.map(DataType::External),
    ))
    .with_span()
    .parse_next(input)
    .map(Into::into)
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: start..input.location(),
            cause,
        })
    })
}

fn parse_basic<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    alt((
        dispatch! {
            take_while(2.., ('a'..='z', '0'..='9', '&'));
            "bool" => empty.value(DataType::Bool),
            "u8" => empty.value(DataType::U8),
            "u16" => empty.value(DataType::U16),
            "u32" => empty.value(DataType::U32),
            "u64" => empty.value(DataType::U64),
            "u128" => empty.value(DataType::U128),
            "i8" => empty.value(DataType::I8),
            "i16" => empty.value(DataType::I16),
            "i32" => empty.value(DataType::I32),
            "i64" => empty.value(DataType::I64),
            "i128" => empty.value(DataType::I128),
            "f32" => empty.value(DataType::F32),
            "f64" => empty.value(DataType::F64),
            "string" => empty.value(DataType::String),
            "&string" => empty.value(DataType::StringRef),
            "bytes" => empty.value(DataType::Bytes),
            "&bytes" => empty.value(DataType::BytesRef),
            _ => fail,
        },
        literal("box<string>").value(DataType::BoxString),
        literal("box<bytes>").value(DataType::BoxBytes),
    ))
    .parse_next(input)
}

fn parse_generic<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    dispatch! {
        (take_while(3.., ('a'..='z', '_')).with_span(), token::Angle::OPEN.span());
        (("vec", ref span), ref open) => cut_err((
                parse.map_err(Cause::from),
                token::Angle::CLOSE.span(),
            ))
            .map(|(ty, close)| DataType::Vec(span.into(), (open, &close).into(), Box::new(ty))),
        (("hash_map", ref span), ref open) => cut_err((
                parse.map_err(Cause::from),
                preceded(space0, token::Comma::VALUE.span()),
                preceded(space0, parse.map_err(Cause::from)),
                token::Angle::CLOSE.span(),
            ))
            .map(|(key, comma, value, close)| DataType::HashMap(
                span.into(),
                (open, &close).into(),
                comma.into(),
                Box::new((key, value)),
            )),
        (("hash_set", ref span), ref open) => cut_err((
                parse.map_err(Cause::from),
                token::Angle::CLOSE.span(),
            ))
            .map(|(ty, close)| DataType::HashSet(span.into(), (open, &close).into(), Box::new(ty))),
        (("option", ref span), ref open) => cut_err((
                parse.map_err(Cause::from),
                token::Angle::CLOSE.span(),
            ))
            .map(|(ty, close)| DataType::Option(span.into(), (open, &close).into(), Box::new(ty))),
        (("non_zero", ref span), ref open) => cut_err((
                parse.map_err(Cause::from),
                token::Angle::CLOSE.span(),
            ))
            .map(|(ty, close)| DataType::NonZero(span.into(), (open, &close).into(), Box::new(ty))),
        _ => fail,
    }
    .parse_next(input)
}

fn parse_tuple<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    (
        token::Parenthesis::OPEN.span(),
        cut_err((
            separated(
                0..,
                ws(parse.map_err(Cause::from)),
                ws(token::Comma::VALUE.span()),
            ),
            ws(token::Parenthesis::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(paren_open, (ty, paren_close))| {
            DataType::Tuple((paren_open, paren_close).into(), ty)
        })
}

fn parse_array<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    (
        token::Bracket::OPEN.span(),
        cut_err((
            ws(parse.map_err(Cause::from)),
            ws(token::Semicolon::VALUE.span()),
            ws(dec_uint),
            ws(token::Bracket::CLOSE.span()),
        )),
    )
        .parse_next(input)
        .map(|(bracket_open, (ty, semicolon, size, bracket_close))| {
            DataType::Array(
                (bracket_open, bracket_close).into(),
                Box::new(ty),
                semicolon.into(),
                size,
            )
        })
}

fn parse_external<'i>(input: &mut Input<'i>) -> Result<ExternalType<'i>, Cause> {
    (
        opt(terminated(
            separated(
                1..,
                imports::parse_segment.map_err(Cause::from),
                token::DoubleColon::VALUE,
            ),
            token::DoubleColon::VALUE,
        ))
        .map(Option::unwrap_or_default),
        parse_external_name,
        opt((
            token::Angle::OPEN.span(),
            cut_err((
                separated(1.., ws(parse.map_err(Cause::from)), ws(token::Comma::VALUE)),
                ws(token::Angle::CLOSE.span()),
            )),
        )),
    )
        .parse_next(input)
        .map(|(path, name, generics)| {
            let (angle, generics) = match generics {
                Some((open, (generics, close))) => (Some((open, close).into()), generics),
                None => (None, Vec::default()),
            };
            ExternalType {
                path,
                name,
                angle,
                generics,
            }
        })
}

fn parse_external_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('A'..='Z'),
        take_while(0.., ('a'..='z', 'A'..='Z', '0'..='9', '_')),
    )
        .recognize()
        .with_span()
        .parse_next(input)
        .map(Into::into)
}
