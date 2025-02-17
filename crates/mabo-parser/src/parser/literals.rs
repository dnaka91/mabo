use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::{dec_uint, digit1, multispace1},
    combinator::{
        alt, cut_err, delimited, fail, opt, peek, preceded, repeat, separated, terminated,
    },
    dispatch,
    error::ErrMode,
    stream::Location,
    token::{any, one_of, take_till, take_while},
};

use super::{Input, Result, ws};
use crate::{
    Literal, LiteralValue, highlight,
    token::{self, Delimiter, Punctuation},
};

/// Encountered an invalid literal declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse literal value"),
    code(mabo::parse::literal),
    help(
        "Expected literal value declaration in either of the forms:\n`{}` or `{}` for \
         booleans\n`{}` for numbers\n`{}` for floating point numbers\n`{}` for strings\nor `{}` \
         for bytes",
        highlight::sample("true"),
        highlight::sample("false"),
        highlight::sample("1, 2, 3, ..."),
        highlight::sample("1.2, 1.0e5, ..."),
        highlight::sample("\"...\""),
        highlight::sample("[...]"),
    )
)]
#[rename(ParseLiteralError)]
pub struct ParseError {
    /// Source location of the whole literal.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a literal declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseLiteralCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(usize),
    /// Found a reference value, which is not allowed.
    #[err(
        msg("Found a reference value"),
        code(mabo::parse::literal::reference),
        help(
            "References are not allowed for literals. Try removing the ampersand ({})",
            highlight::value("&"),
        )
    )]
    FoundReference {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Encountered an invalid integer literal.
    #[err(
        msg("Invalid integer literal"),
        code(mabo::parse::literal::int),
        help("Integers must only consist of digits ({})", highlight::value("0-9"))
    )]
    InvalidInt {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
    },
    /// Failed to parse value as integer.
    #[external]
    ParseInt {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
        /// Inner error that cause the failure.
        cause: std::num::ParseIntError,
    },
}

pub(super) fn parse(input: &mut Input<'_>) -> Result<Literal, ParseError> {
    let start = input.current_token_start();

    dispatch! {
        peek(any);
        't' => parse_true.map(LiteralValue::Bool),
        'f' => parse_false.map(LiteralValue::Bool),
        '+' | '-' | '0'..='9' => cut_err(alt((
            parse_float.map(LiteralValue::Float),
            parse_int.map(LiteralValue::Int)
        ))),
        '"' => parse_string.map(LiteralValue::String),
        '[' => parse_bytes.map(LiteralValue::Bytes),
        '&' => fail,
        _ => fail,
    }
    .with_span()
    .parse_next(input)
    .map(Literal::from)
    .map_err(|e| {
        e.map(|cause| ParseError {
            at: start..start,
            cause,
        })
    })
}

fn parse_true(input: &mut Input<'_>) -> Result<bool, Cause> {
    "true".value(true).parse_next(input)
}

fn parse_false(input: &mut Input<'_>) -> Result<bool, Cause> {
    "false".value(false).parse_next(input)
}

fn parse_float(input: &mut Input<'_>) -> Result<f64, Cause> {
    (
        opt(one_of(['+', '-'])),
        (digit1, '.', digit1),
        opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), cut_err(digit1))),
    )
        .take()
        .parse_to()
        .parse_next(input)
}

fn parse_int(input: &mut Input<'_>) -> Result<i128, Cause> {
    (opt(one_of(['+', '-'])), digit1)
        .take()
        .parse_to()
        .parse_next(input)
        .map_err(|e: ErrMode<_>| {
            e.map(|()| Cause::InvalidInt {
                at: input.current_token_start(),
            })
        })
}

fn parse_string(input: &mut Input<'_>) -> Result<String, Cause> {
    preceded(
        '"',
        cut_err(terminated(
            repeat(0.., parse_fragment).fold(String::new, |mut acc, fragment| {
                match fragment {
                    Fragment::Literal(s) => acc.push_str(s),
                    Fragment::EscapedChar(c) => acc.push(c),
                    Fragment::EscapedWhitespace => {}
                }
                acc
            }),
            '"',
        )),
    )
    .parse_next(input)
}

enum Fragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWhitespace,
}

fn parse_fragment<'i>(input: &mut Input<'i>) -> Result<Fragment<'i>, Cause> {
    alt((
        parse_string_literal.map(Fragment::Literal),
        parse_string_escaped_char.map(Fragment::EscapedChar),
        parse_string_escaped_whitespace.map(|_| Fragment::EscapedWhitespace),
    ))
    .parse_next(input)
}

fn parse_string_literal<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    take_till(1.., ['"', '\\'])
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

fn parse_string_escaped_char(input: &mut Input<'_>) -> Result<char, Cause> {
    preceded(
        '\\',
        dispatch!(
            peek(any);
            'n' => 'n'.value('\n'),
            'r' => 'r'.value('\r'),
            't' => 't'.value('\t'),
            'b' => 'b'.value('\u{08}'),
            'f' => 'f'.value('\u{0c}'),
            '\\' => '\\'.value('\\'),
            '\0' => '\0'.value('\0'),
            '/' => '/'.value('/'),
            '"' => '"'.value('"'),
            'u' => parse_string_unicode,
            _ => fail,
        ),
    )
    .parse_next(input)
}

fn parse_string_unicode(input: &mut Input<'_>) -> Result<char, Cause> {
    preceded(
        'u',
        cut_err(delimited(
            token::Brace::OPEN,
            take_while(1..=6, ('0'..='9', 'a'..='f', 'A'..='F')),
            token::Brace::CLOSE,
        )),
    )
    .try_map(|hex| u32::from_str_radix(hex, 16))
    .verify_map(std::char::from_u32)
    .parse_next(input)
}

fn parse_string_escaped_whitespace<'i>(input: &mut Input<'i>) -> Result<&'i str, Cause> {
    preceded('\\', multispace1).parse_next(input)
}

fn parse_bytes(input: &mut Input<'_>) -> Result<Vec<u8>, Cause> {
    preceded(
        token::Bracket::OPEN,
        cut_err(terminated(
            separated(1.., ws(dec_uint::<_, u8, _>), ws(token::Comma::VALUE)),
            (opt(ws(token::Comma::VALUE)), ws(token::Bracket::CLOSE)),
        )),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_found_reference() {
        let err = ParseError {
            at: (22..29),
            cause: Cause::FoundReference { at: 22 },
        };

        println!(
            "{:?}",
            miette::Report::from(err).with_source_code("const VALUE: string = &\"test\";")
        );
    }
}
