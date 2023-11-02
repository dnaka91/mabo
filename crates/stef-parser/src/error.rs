//! Possible errors that can occur when parsing schema files.
//!
//!
//! The root element is the [`ParseSchemaError`], which forms a tree of errors down to a specific
//! error that caused parsing to fail.

#![allow(missing_docs, clippy::module_name_repetitions)]

use std::{
    error::Error,
    fmt::{self, Display},
};

use miette::{Diagnostic, NamedSource};
use winnow::error::ErrorKind;

pub use crate::parser::{
    ParseAliasCause, ParseAliasError, ParseAttributeCause, ParseAttributeError, ParseCommentCause,
    ParseCommentError, ParseConstCause, ParseConstError, ParseEnumCause, ParseEnumError,
    ParseFieldsCause, ParseFieldsError, ParseGenericsCause, ParseGenericsError, ParseIdCause,
    ParseIdError, ParseImportCause, ParseImportError, ParseLiteralCause, ParseLiteralError,
    ParseModuleCause, ParseModuleError, ParseStructCause, ParseStructError, ParseTypeCause,
    ParseTypeError,
};

/// Reason why a `STEF` schema definition was invalid.
#[derive(Debug)]
pub struct ParseSchemaError {
    pub(crate) source_code: NamedSource,
    pub cause: ParseSchemaCause,
}

impl Error for ParseSchemaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.source()
    }
}

impl Display for ParseSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(f)
    }
}

impl Diagnostic for ParseSchemaError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.cause.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_code)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.cause.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.cause.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.cause.diagnostic_source()
    }
}

#[derive(Debug, Diagnostic)]
pub enum ParseSchemaCause {
    Parser(ErrorKind),
    #[diagnostic(transparent)]
    Definition(ParseDefinitionError),
}

impl Error for ParseSchemaCause {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parser(kind) => kind.source(),
            Self::Definition(inner) => inner.source(),
        }
    }
}

impl Display for ParseSchemaCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(kind) => kind.fmt(f),
            Self::Definition(inner) => inner.fmt(f),
        }
    }
}

impl From<ParseDefinitionError> for ParseSchemaCause {
    fn from(value: ParseDefinitionError) -> Self {
        Self::Definition(value)
    }
}

impl<I> winnow::error::ParserError<I> for ParseSchemaCause {
    fn from_error_kind(_: &I, kind: winnow::error::ErrorKind) -> Self {
        Self::Parser(kind)
    }

    fn append(self, _: &I, _: winnow::error::ErrorKind) -> Self {
        self
    }
}

/// Reason why a single definition was invalid.
#[derive(Debug, Diagnostic)]
pub enum ParseDefinitionError {
    Parser(ErrorKind),
    #[diagnostic(transparent)]
    Comment(ParseCommentError),
    #[diagnostic(transparent)]
    Attribute(ParseAttributeError),
    #[diagnostic(transparent)]
    Module(ParseModuleError),
    #[diagnostic(transparent)]
    Struct(ParseStructError),
    #[diagnostic(transparent)]
    Enum(ParseEnumError),
    #[diagnostic(transparent)]
    Const(ParseConstError),
    #[diagnostic(transparent)]
    Alias(ParseAliasError),
    #[diagnostic(transparent)]
    Import(ParseImportError),
}

impl Error for ParseDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parser(kind) => kind.source(),
            Self::Comment(inner) => inner.source(),
            Self::Attribute(inner) => inner.source(),
            Self::Module(inner) => inner.source(),
            Self::Struct(inner) => inner.source(),
            Self::Enum(inner) => inner.source(),
            Self::Const(inner) => inner.source(),
            Self::Alias(inner) => inner.source(),
            Self::Import(inner) => inner.source(),
        }
    }
}

impl Display for ParseDefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(kind) => kind.fmt(f),
            Self::Comment(inner) => inner.fmt(f),
            Self::Attribute(inner) => inner.fmt(f),
            Self::Module(inner) => inner.fmt(f),
            Self::Struct(inner) => inner.fmt(f),
            Self::Enum(inner) => inner.fmt(f),
            Self::Const(inner) => inner.fmt(f),
            Self::Alias(inner) => inner.fmt(f),
            Self::Import(inner) => inner.fmt(f),
        }
    }
}

impl From<ParseCommentError> for ParseDefinitionError {
    fn from(value: ParseCommentError) -> Self {
        Self::Comment(value)
    }
}

impl From<ParseAttributeError> for ParseDefinitionError {
    fn from(value: ParseAttributeError) -> Self {
        Self::Attribute(value)
    }
}

impl From<ParseModuleError> for ParseDefinitionError {
    fn from(value: ParseModuleError) -> Self {
        Self::Module(value)
    }
}

impl From<ParseStructError> for ParseDefinitionError {
    fn from(value: ParseStructError) -> Self {
        Self::Struct(value)
    }
}

impl From<ParseEnumError> for ParseDefinitionError {
    fn from(value: ParseEnumError) -> Self {
        Self::Enum(value)
    }
}

impl From<ParseConstError> for ParseDefinitionError {
    fn from(value: ParseConstError) -> Self {
        Self::Const(value)
    }
}

impl From<ParseAliasError> for ParseDefinitionError {
    fn from(value: ParseAliasError) -> Self {
        Self::Alias(value)
    }
}

impl From<ParseImportError> for ParseDefinitionError {
    fn from(value: ParseImportError) -> Self {
        Self::Import(value)
    }
}

impl<I> winnow::error::ParserError<I> for ParseDefinitionError {
    fn from_error_kind(_: &I, kind: winnow::error::ErrorKind) -> Self {
        Self::Parser(kind)
    }

    fn append(self, _: &I, _: winnow::error::ErrorKind) -> Self {
        self
    }
}
