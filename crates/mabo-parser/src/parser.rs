use std::ops::Range;

use winnow::{
    ascii::{multispace0, newline, space0},
    combinator::{cut_err, fail, opt, peek, preceded, repeat, terminated, trace},
    dispatch,
    error::{ErrMode, ParserError},
    prelude::*,
    stream::{AsChar, Compare, Location, Stream, StreamIsPartial},
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
    Definition, Schema,
    error::{ParseDefinitionError, ParseSchemaCause},
    ext::ParserExt,
    punctuated::Punctuated,
    token::Delimiter,
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
mod types; /*  */

type Input<'i> = winnow::LocatingSlice<&'i str>;
type Result<T, E = ParseSchemaCause> = winnow::ModalResult<T, E>;

pub(crate) fn parse_schema<'i>(input: &mut Input<'i>) -> Result<Schema<'i>, ParseSchemaCause> {
    let source = *input.as_ref();

    trace(
        "schema",
        (
            opt(terminated(
                ws(comments::parse.map_err2(Into::into)),
                newline,
            )),
            terminated(
                repeat(0.., parse_definition.map_err2(Into::into)),
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
        ws(comments::parse.map_err2(Into::into)),
        ws(attributes::parse.map_err2(Into::into)),
        preceded(
            space0,
            dispatch! {
                peek(any);
                'm' => modules::parse.map(Definition::Module).map_err2(Into::into),
                's' => structs::parse.map(Definition::Struct).map_err2(Into::into),
                'e' => enums::parse.map(Definition::Enum).map_err2(Into::into),
                'c' => consts::parse.map(Definition::Const).map_err2(Into::into),
                't' => aliases::parse.map(Definition::TypeAlias).map_err2(Into::into),
                'u' => imports::parse.map(Definition::Import).map_err2(Into::into),
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
    use winnow::{Parser, ascii::dec_uint, combinator::preceded, error::ErrMode, stream::Location};

    use super::{Input, Result};
    use crate::{Id, highlight};

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
        Parser(usize),
    }

    pub(super) fn parse(input: &mut Input<'_>) -> Result<Id, ParseError> {
        preceded('@', dec_uint)
            .with_span()
            .parse_next(input)
            .map(Id::from)
            .map_err(|e: ErrMode<_>| {
                e.map(|()| ParseError {
                    at: input.current_token_start()..input.current_token_start(),
                    cause: Cause::Parser(input.current_token_start()),
                })
            })
    }
}

mod comments {
    use std::ops::Range;

    use mabo_derive::{ParserError, ParserErrorCause};
    use winnow::{
        Parser,
        ascii::space0,
        combinator::{delimited, preceded, repeat},
        error::ErrMode,
        stream::Stream,
        token::take_till,
    };

    use super::{Input, Result};
    use crate::{Comment, CommentLine, highlight, location};

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
        Parser(usize),
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
        .map_err(|e: ErrMode<_>| {
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

pub fn punctuate<I, O, P, E, F, G>(mut f: F, mut g: G) -> impl ModalParser<I, Punctuated<O, P>, E>
where
    I: Stream,
    E: ParserError<I>,
    F: ModalParser<I, (O, P), E>,
    G: ModalParser<I, (O, Option<P>), E>,
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

                    values.push((o, range));
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
                    return Ok(Punctuated::new(values, (o, range)));
                }
                Err(e) => return Err(e),
            }
        }
    })
}

pub fn surround<I, O, D, E, F>(f: F) -> impl ModalParser<I, (D, O), E>
where
    I: Compare<char> + Location + Stream + StreamIsPartial,
    I::Token: Clone + AsChar,
    D: Delimiter + From<(Range<usize>, Range<usize>)>,
    E: ParserError<I>,
    F: ModalParser<I, O, E>,
{
    let mut parser = (D::OPEN.span(), cut_err((f, ws(D::CLOSE.span()))));

    trace("surround", move |i: &mut I| {
        parser
            .parse_next(i)
            .map(|(open, (o, close))| ((open, close).into(), o))
    })
}
