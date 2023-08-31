use std::ops::Range;

use stef_derive::{ParserError, ParserErrorCause};
use winnow::{
    ascii::{alphanumeric0, dec_uint, space0},
    combinator::{
        alt, cut_err, fail, opt, preceded, separated1, separated_pair, success, terminated,
    },
    dispatch,
    error::ErrorKind,
    stream::Location,
    token::{one_of, tag, take_while},
    Parser,
};

use super::{imports, ws, Input, ParserExt, Result};
use crate::{highlight, DataType, ExternalType};

/// Encountered an invalid type definition.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse type definition"),
    code(stef::parse::type_def),
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
    Parser(ErrorKind),
    /// Invalid type declaration.
    #[forward]
    Type(Box<ParseError>),
    /// Invalid path segment.
    #[forward]
    Segment(Box<imports::Cause>),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, ParseError> {
    let start = input.location();

    alt((
        parse_basic,
        parse_generic,
        parse_tuple,
        parse_array,
        parse_external.map(DataType::External),
    ))
    .parse_next(input)
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
            "bool" => success(DataType::Bool),
            "u8" => success(DataType::U8),
            "u16" => success(DataType::U16),
            "u32" => success(DataType::U32),
            "u64" => success(DataType::U64),
            "u128" => success(DataType::U128),
            "i8" => success(DataType::I8),
            "i16" => success(DataType::I16),
            "i32" => success(DataType::I32),
            "i64" => success(DataType::I64),
            "i128" => success(DataType::I128),
            "f32" => success(DataType::F32),
            "f64" => success(DataType::F64),
            "string" => success(DataType::String),
            "&string" => success(DataType::StringRef),
            "bytes" => success(DataType::Bytes),
            "&bytes" => success(DataType::BytesRef),
            _ => fail,
        },
        tag("box<string>").value(DataType::BoxString),
        tag("box<bytes>").value(DataType::BoxBytes),
    ))
    .parse_next(input)
}

fn parse_generic<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    terminated(
        dispatch! {
            terminated(take_while(3.., ('a'..='z', '_')), '<');
            "vec" => cut_err(parse.map_err(Cause::from))
                .map(|t| DataType::Vec(Box::new(t))),
            "hash_map" => cut_err(separated_pair(
                    parse.map_err(Cause::from),
                    (',', space0),
                    parse.map_err(Cause::from),
                ))
                .map(|kv| DataType::HashMap(Box::new(kv))),
            "hash_set" => cut_err(parse.map_err(Cause::from))
                .map(|t| DataType::HashSet(Box::new(t))),
            "option" => cut_err(parse.map_err(Cause::from))
                .map(|t| DataType::Option(Box::new(t))),
            "non_zero" => cut_err(parse.map_err(Cause::from))
                .map(|t| DataType::NonZero(Box::new(t))),
            _ => fail,
        },
        '>',
    )
    .parse_next(input)
}

fn parse_tuple<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    preceded(
        '(',
        cut_err(terminated(
            separated1(ws(parse.map_err(Cause::from)), ws(',')),
            ws(')'),
        )),
    )
    .parse_next(input)
    .map(DataType::Tuple)
}

fn parse_array<'i>(input: &mut Input<'i>) -> Result<DataType<'i>, Cause> {
    preceded(
        '[',
        cut_err(terminated(
            separated_pair(ws(parse.map_err(Cause::from)), ws(';'), ws(dec_uint)),
            ws(']'),
        )),
    )
    .parse_next(input)
    .map(|(t, size)| DataType::Array(Box::new(t), size))
}

fn parse_external<'i>(input: &mut Input<'i>) -> Result<ExternalType<'i>, Cause> {
    (
        opt(terminated(
            separated1(imports::parse_segment.map_err(Cause::from), "::"),
            "::",
        ))
        .map(Option::unwrap_or_default),
        parse_external_name,
        opt(preceded(
            '<',
            cut_err(terminated(
                separated1(ws(parse.map_err(Cause::from)), ws(',')),
                ws('>'),
            )),
        ))
        .map(Option::unwrap_or_default),
    )
        .parse_next(input)
        .map(|(path, name, generics)| ExternalType {
            path,
            name,
            generics,
        })
}

fn parse_external_name<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    (one_of('A'..='Z'), alphanumeric0)
        .recognize()
        .parse_next(input)
}
