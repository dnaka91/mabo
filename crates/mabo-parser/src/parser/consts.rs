use std::ops::Range;

use mabo_derive::{ParserError, ParserErrorCause};
use winnow::{
    Parser,
    ascii::{space0, space1},
    combinator::{cut_err, preceded, terminated},
    error::ErrMode,
    stream::{Location, Stream},
    token::{one_of, take_while},
};

use super::{Input, ParserExt, Result, literals, types};
use crate::{Comment, Const, Name, highlight, location, token};

/// Encountered an invalid `const` declaration.
#[derive(Debug, ParserError)]
#[err(
    msg("Failed to parse const declaration"),
    code(mabo::parse::const_def),
    help(
        "Expected const declaration in the form `{}`",
        highlight::sample("const <NAME>: <type> = <literal>;"),
    )
)]
#[rename(ParseConstError)]
pub struct ParseError {
    /// Source location of the whole const.
    #[err(label("In this declaration"))]
    pub at: Range<usize>,
    /// Specific cause of the error.
    pub cause: Cause,
}

/// Specific reason why a `const` declaration was invalid.
#[derive(Debug, ParserErrorCause)]
#[rename(ParseConstCause)]
pub enum Cause {
    /// Non-specific general parser error.
    Parser(usize),
    #[err(
        msg("Unexpected character"),
        code(mabo::parse::const_def::char),
        help("Expected a `{}` here", highlight::value(expected))
    )]
    /// Encountered an unexpected character.
    UnexpectedChar {
        /// Source location of the character.
        #[err(label("Problematic character"))]
        at: usize,
        /// The character that was expected instead.
        expected: char,
    },
    /// Defined name is not considered valid.
    #[err(
        msg("Invalid const name"),
        code(mabo::parse::const_def::invalid_name),
        help(
            "Const names must start with an uppercase letter ({}), followed by zero or more \
             uppercase alphanumeric characters or underscores ({})",
            highlight::value("A-Z"),
            highlight::value("A-Z, 0-9, _"),
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
    /// Invalid const value literal.
    #[forward]
    Literal(literals::ParseError),
}

pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Const<'i>, ParseError> {
    let start = input.checkpoint();

    (
        terminated(token::Const::parser(), space1),
        cut_err((
            parse_name,
            token::Colon::parser(),
            preceded(space0, types::parse.map_err2(Cause::from)),
            preceded(space0, token::Equal::parser()),
            preceded(space0, literals::parse.map_err2(Cause::from)),
            token::Semicolon::parser()
                .map_err_loc(|at, ()| Cause::UnexpectedChar { at, expected: ';' }),
        )),
    )
        .parse_next(input)
        .map(
            |(keyword, (name, colon, ty, equal, value, semicolon))| Const {
                comment: Comment::default(),
                keyword,
                name,
                colon,
                ty,
                equal,
                value,
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

pub(super) fn parse_name<'i>(input: &mut Input<'i>) -> Result<Name<'i>, Cause> {
    (
        one_of('A'..='Z'),
        take_while(0.., ('A'..='Z', '0'..='9', '_')),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_const() {
        let err = ParseError {
            at: (0..21),
            cause: Cause::InvalidName { at: 6 },
        };
        println!(
            "{:?}",
            miette::Report::from(err).with_source_code("const vALUE: u32 = 1;")
        );
    }
}
