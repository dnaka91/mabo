//! Tokens of the Mabo schema language that represent keywords, punctuation and delimiters.

use std::{
    fmt::{self, Display},
    ops::Range,
};

use mabo_derive::Debug;
use winnow::{
    Parser,
    error::ParserError,
    stream::{Compare, Location, Stream, StreamIsPartial},
};

use crate::{Print, Span, Spanned};

macro_rules! define_keywords {
    ($(#[$doc:meta] $name:ident $token:literal)*) => {
        $(
            #[$doc]
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct $name {
                span: Span,
            }

            impl Spanned for $name {
                fn span(&self) -> Span {
                    self.span
                }
            }

            impl Print for $name {
                fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
                    Self::indent(f, level)?;
                    f.write_str($token)
                }
            }

            impl Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.print(f, 0)
                }
            }

            impl<'a> From<Range<usize>> for $name {
                fn from(span: Range<usize>) -> Self {
                    Self {
                        span: span.into(),
                    }
                }
            }

            impl $name {
                /// Literal raw name of this keyword.
                pub const NAME: &'static str = $token;

                #[inline]
                pub(crate) fn parser<'a, I, E>() -> impl Parser<I, Self, E> + use<I, E>
                where
                    I: Compare<&'a str> + Location + Stream + StreamIsPartial,
                    E: ParserError<I>,
                {
                    |i: &mut I| Self::NAME.span().output_into().parse_next(i)
                }
            }
        )*
    };
}

/// A textual element that is used to separate repeated elements in the schema, or act as a form of
/// prefix.
pub trait Punctuation {
    /// String that makes up the punctuation element.
    const VALUE: &'static str;
}

macro_rules! define_punctuation {
    ($(#[$doc:meta] $name:ident $token:literal)*) => {
        $(
            #[$doc]
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct $name {
                span: Span,
            }

            impl Spanned for $name {
                fn span(&self) -> Span {
                    self.span
                }
            }

            impl<'a> From<Range<usize>> for $name {
                fn from(span: Range<usize>) -> Self {
                    Self {
                        span: span.into(),
                    }
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        span: Span {
                            start: 0,
                            end: 0,
                        }
                    }
                }
            }

            impl Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str($token)
                }
            }

            impl Punctuation for  $name {
                /// Literal raw string of this punctuation.
                const VALUE: &'static str = $token;
            }

            impl $name {
                #[inline]
                pub(crate) fn parser<'a, I, E>() -> impl Parser<I, Self, E> + use<I, E>
                where
                    I: Compare<&'a str> + Location + Stream + StreamIsPartial,
                    E: ParserError<I>,
                {
                    |i: &mut I| Self::VALUE.span().output_into().parse_next(i)
                }
            }
        )*
    };
}

/// Delimiters surround other elements with an opening and closing character.
pub trait Delimiter {
    /// Opening character of the delimiter.
    const OPEN: char;
    /// Closing character of the delimiter.
    const CLOSE: char;

    /// Get the location of the opening token.
    fn open(&self) -> Span;
    /// Get the location of the closing token.
    fn close(&self) -> Span;
    /// Get a combined span that goes from the open to the close token.
    fn range(&self) -> Span;
}

macro_rules! define_delimiters {
    ($(#[$doc:meta] $name:ident $token_open:literal $token_close:literal)*) => {
        $(
            #[$doc]
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct $name {
                open: Span,
                close: Span,
            }

            impl<'a> From<(Range<usize>, Range<usize>)> for $name {
                fn from((open, close): (Range<usize>, Range<usize>)) -> Self {
                    Self {
                        open: open.into(),
                        close: close.into(),
                    }
                }
            }

            impl<'a> From<(&Range<usize>, &Range<usize>)> for $name {
                fn from((open, close): (&Range<usize>, &Range<usize>)) -> Self {
                    Self {
                        open: open.into(),
                        close: close.into(),
                    }
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        open: Span {
                            start: 0,
                            end: 0,
                        },
                        close: Span {
                            start: 0,
                            end: 0,
                        }
                    }
                }
            }

            impl Delimiter for $name {
                const OPEN: char = $token_open;
                const CLOSE: char = $token_close;

                fn open(&self) -> Span {
                    self.open
                }

                fn close(&self) -> Span {
                    self.close
                }

                fn range(&self) -> Span {
                    Span{
                        start: self.open.start,
                        end: self.close.end,
                    }
                }
            }
        )*
    };
}

define_keywords! {
    /// The `mod` keyword.
    Mod "mod"
    /// The `struct` keyword.
    Struct "struct"
    /// The `enum` keyword.
    Enum "enum"
    /// The `const` keyword.
    Const "const"
    /// The `type` keyword.
    Type "type"
    /// The `use` keyword.
    Use "use"
}

define_punctuation! {
    /// Comma `,` separator, usually used to separate fields or enum variants.
    Comma ","
    /// Colon `:` separator, as separator between field names and their type.
    Colon ":"
    /// Semicolon `;` separator, as terminator for type aliases.
    Semicolon ";"
    /// Pound `#` punctuation, as start for an attribute block.
    Pound "#"
    /// Double colon `::` separator, as path separator for type paths.
    DoubleColon "::"
    /// Equal sign `=` separator, used in type aliases.
    Equal "="
}

define_delimiters! {
    /// Braces `{`...`}`, to surround module content and named fields.
    Brace '{' '}'
    /// Brackets `[`...`]`, to surround byte array literals.
    Bracket '[' ']'
    /// Parenthesis `(`...`)`, to surround unnamed fields and tuples.
    Parenthesis '(' ')'
    /// Angles `<`...`>`, to surround generics.
    Angle '<' '>'
}
