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

use miette::Diagnostic;
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
#[derive(Debug, Diagnostic)]
pub enum ParseSchemaError {
    Parser(ErrorKind),
    #[diagnostic(transparent)]
    Definition(ParseDefinitionError),
}

impl Error for ParseSchemaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parser(kind) => kind.source(),
            Self::Definition(inner) => inner.source(),
        }
    }
}

impl Display for ParseSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(kind) => kind.fmt(f),
            Self::Definition(inner) => inner.fmt(f),
        }
    }
}

impl From<ParseDefinitionError> for ParseSchemaError {
    fn from(value: ParseDefinitionError) -> Self {
        Self::Definition(value)
    }
}

impl<I> winnow::error::ParserError<I> for ParseSchemaError {
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
