use std::ops::Range;

use winnow::{
    ascii::{multispace0, newline, space0},
    combinator::{fail, opt, peek, preceded, repeat, terminated, trace},
    dispatch,
    error::{ErrMode, ParserError},
    prelude::*,
    stream::{AsChar, Stream, StreamIsPartial},
    token::any,
};

pub use self::{
    aliases::{Cause as ParseAliasCause, ParseError as ParseAliasError},
    attributes::{Cause as ParseAttributeCause, ParseError as ParseAttributeError},
    comments::{Cause as ParseCommentCause, ParseError as ParseCommentError},
    consts::{Cause as ParseConstCause, ParseError as ParseConstError},
    enums::{Cause as ParseEnumCause, ParseError as ParseEnumError},
    fields::{Cause as ParseFieldsCause, ParseError as ParseFieldsError},
    generics::{Cause as ParseGenericsCause, ParseError as ParseGenericsError},
    ids::{Cause as ParseIdCause, ParseError as ParseIdError},
    imports::{Cause as ParseImportCause, ParseError as ParseImportError},
    literals::{Cause as ParseLiteralCause, ParseError as ParseLiteralError},
    modules::{Cause as ParseModuleCause, ParseError as ParseModuleError},
    structs::{Cause as ParseStructCause, ParseError as ParseStructError},
    types::{Cause as ParseTypeCause, ParseError as ParseTypeError},
};
use crate::{
    error::{ParseDefinitionError, ParseSchemaCause},
    ext::ParserExt,
    punctuated::Punctuated,
    Definition, Schema,
};

mod aliases;
mod attributes;
mod consts;
mod enums;
mod fields;
mod generics;
mod imports;
mod literals;
mod modules;
mod structs;
mod types;

type Input<'i> = winnow::Located<&'i str>;
type Result<T, E = ParseSchemaCause> = winnow::PResult<T, E>;

pub(crate) fn parse_schema<'i>(input: &mut Input<'i>) -> Result<Schema<'i>, ParseSchemaCause> {
    let source = *input.as_ref();

    trace(
        "schema",
        (
            opt(terminated(ws(comments::parse.map_err(Into::into)), newline)),
            terminated(
                repeat(0.., parse_definition.map_err(Into::into)),
                multispace0,
            ),
        ),
    )
    .parse_next(input)
    .map(|(comment, definitions)| Schema {
        path: None,
        source,
        comment: comment.unwrap_or_default(),
        definitions,
    })
}

fn parse_definition<'i>(input: &mut Input<'i>) -> Result<Definition<'i>, ParseDefinitionError> {
    (
        ws(comments::parse.map_err(Into::into)),
        ws(attributes::parse.map_err(Into::into)),
        preceded(
            space0,
            dispatch! {
                peek(any);
                'm' => modules::parse.map(Definition::Module).map_err(Into::into),
                's' => structs::parse.map(Definition::Struct).map_err(Into::into),
                'e' => enums::parse.map(Definition::Enum).map_err(Into::into),
                'c' => consts::parse.map(Definition::Const).map_err(Into::into),
                't' => aliases::parse.map(Definition::TypeAlias).map_err(Into::into),
                'u' => imports::parse.map(Definition::Import).map_err(Into::into),
                _ => fail,
            },
        ),
    )
        .parse_next(input)
        .map(|(comment, attributes, def)| def.with_comment(comment).with_attributes(attributes))
}

mod ids {
    use std::ops::Range;

    use mabo_derive::{ParserError, ParserErrorCause};
    use winnow::{
        ascii::dec_uint, combinator::preceded, error::ErrorKind, stream::Location, Parser,
    };

    use super::{Input, Result};
    use crate::{highlight, Id};

    /// Encountered an invalid `@...` id declaration.
    #[derive(Debug, ParserError)]
    #[err(
        msg("Failed to parse id declaration"),
        code(mabo::parse::id),
        help("Expected id declaration in the form `{}`", highlight::sample("@..."))
    )]
    #[rename(ParseIdError)]
    pub struct ParseError {
        /// Source location of the whole id.
        #[err(label("In this declaration"))]
        pub at: Range<usize>,
        /// Specific cause of the error.
        pub cause: Cause,
    }

    /// Specific reason why a `@...` id declaration was invalid.
    #[derive(Debug, ParserErrorCause)]
    pub enum Cause {
        /// Non-specific general parser error.
        Parser(ErrorKind, usize),
    }

    pub(super) fn parse(input: &mut Input<'_>) -> Result<Id, ParseError> {
        preceded('@', dec_uint)
            .with_span()
            .parse_next(input)
            .map(Id::from)
            .map_err(|e| {
                e.map(|e: ErrorKind| ParseError {
                    at: input.location()..input.location(),
                    cause: Cause::Parser(e, input.location()),
                })
            })
    }
}

mod comments {
    use std::ops::Range;

    use mabo_derive::{ParserError, ParserErrorCause};
    use winnow::{
        ascii::space0,
        combinator::{delimited, preceded, repeat},
        error::ErrorKind,
        stream::Stream,
        token::take_till,
        Parser,
    };

    use super::{Input, Result};
    use crate::{highlight, location, Comment, CommentLine};

    /// Encountered an invalid `/// ...` comment declaration.
    #[derive(Debug, ParserError)]
    #[err(
        msg("Failed to parse comment declaration"),
        code(mabo::parse::comment),
        help(
            "Expected comment declaration in the form `{}`",
            highlight::sample("/// ..."),
        )
    )]
    #[rename(ParseCommentError)]
    pub struct ParseError {
        /// Source location of the whole comment.
        #[err(label("In this declaration"))]
        pub at: Range<usize>,
        /// Specific cause of the error.
        pub cause: Cause,
    }

    /// Specific reason why a `/// ...` comment declaration was invalid.
    #[derive(Debug, ParserErrorCause)]
    pub enum Cause {
        /// Non-specific general parser error.
        Parser(ErrorKind, usize),
    }

    pub(super) fn parse<'i>(input: &mut Input<'i>) -> Result<Comment<'i>, ParseError> {
        let start = input.checkpoint();

        repeat(
            0..,
            delimited(
                space0,
                preceded(("///", space0), take_till(0.., '\n')).with_span(),
                '\n',
            )
            .map(CommentLine::from),
        )
        .parse_next(input)
        .map(Comment)
        .map_err(|e| {
            e.map(|cause| ParseError {
                at: location::from_until(*input, &start, ['\n']),
                cause,
            })
        })
    }
}

#[inline]
fn ws<F, I, O, E: ParserError<I>>(inner: F) -> impl Parser<I, O, E>
where
    I: Stream + StreamIsPartial,
    <I as Stream>::Token: AsChar + Clone,
    F: Parser<I, O, E>,
{
    trace("ws", preceded(multispace0, inner))
}

pub fn punctuate<I, O, P, E, F, G>(mut f: F, mut g: G) -> impl Parser<I, Punctuated<O, P>, E>
where
    I: Stream,
    P: Copy + From<Range<usize>>,
    E: ParserError<I>,
    F: Parser<I, (O, Range<usize>), E>,
    G: Parser<I, (O, Option<Range<usize>>), E>,
{
    trace("punctuate", move |i: &mut I| {
        let mut values = Vec::new();
        let mut start_prev = None;

        loop {
            let start = i.checkpoint();
            let len = i.eof_offset();

            match f.parse_next(i) {
                Ok((o, range)) => {
                    if i.eof_offset() == len {
                        return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                    }

                    values.push((o, range.into()));
                    start_prev = Some(start);
                }
                Err(ErrMode::Backtrack(_)) => {
                    i.reset(&start);
                    let (o, range) = match g.parse_next(i) {
                        Ok(o) => Ok(o),
                        // Both parsers failed. Lets undo the last successful action from the first
                        // parser and redo it with the second one to get the last value.
                        //
                        // Technically we could just take the parsed data from the first parser and
                        // transform it, but there is no guarantee that the parsers are similar
                        // enough to do this transformation.
                        Err(ErrMode::Backtrack(e)) => {
                            if let Some(start) = start_prev {
                                values.pop();
                                i.reset(&start);
                                g.parse_next(i)
                            } else {
                                Err(ErrMode::Backtrack(e))
                            }
                        }
                        Err(e) => Err(e),
                    }?;
                    return Ok(Punctuated::new(values, (o, range.map(Into::into))));
                }
                Err(e) => return Err(e),
            }
        }
    })
}
