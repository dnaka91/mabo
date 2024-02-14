use std::ops::Range;

use lsp_types::{self as lsp, Diagnostic, Url};
use mabo_compiler::validate;
use mabo_parser::{
    error::{
        ParseAliasCause, ParseAttributeCause, ParseCommentError, ParseConstCause,
        ParseDefinitionError, ParseEnumCause, ParseFieldsCause, ParseFieldsError,
        ParseGenericsError, ParseIdError, ParseImportCause, ParseLiteralCause, ParseModuleCause,
        ParseSchemaCause, ParseSchemaError, ParseStructCause, ParseTypeCause, ParseTypeError,
    },
    Schema,
};

use super::index::Index;

pub fn compile<'a>(file: Url, schema: &'a str, index: &'_ Index) -> Result<Schema<'a>, Diagnostic> {
    let parsed =
        mabo_parser::Schema::parse(schema, None).map_err(|e| parse_schema_diagnostic(index, &e))?;

    mabo_compiler::validate_schema(&parsed)
        .map_err(|e| validate_schema_diagnostic(file, index, e))?;

    Ok(parsed)
}

pub fn simplify<'a>(
    result: &'a Result<Schema<'a>, Diagnostic>,
) -> Result<mabo_compiler::simplify::Schema<'a>, &'a Diagnostic> {
    result.as_ref().map(mabo_compiler::simplify_schema)
}

fn parse_schema_diagnostic(index: &Index, e: &ParseSchemaError) -> Diagnostic {
    match &e.cause {
        ParseSchemaCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseSchemaCause::Comment(e) => parse_comment_diagnostic(index, e),
        ParseSchemaCause::Definition(e) => parse_definition_diagnostic(index, e),
    }
}

fn parse_definition_diagnostic(index: &Index, e: &ParseDefinitionError) -> Diagnostic {
    match e {
        ParseDefinitionError::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseDefinitionError::Comment(e) => parse_comment_diagnostic(index, e),
        ParseDefinitionError::Attribute(e) => match &e.cause {
            ParseAttributeCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseAttributeCause::Literal(e) => match &e.cause {
                ParseLiteralCause::Parser(_, at) => {
                    Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
                }
                ParseLiteralCause::FoundReference { at }
                | ParseLiteralCause::InvalidInt { at }
                | ParseLiteralCause::ParseInt { at, .. } => {
                    Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
                }
            },
        },
        ParseDefinitionError::Module(e) => match &e.cause {
            ParseModuleCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseModuleCause::InvalidName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
            }
            ParseModuleCause::Definition(e) => parse_definition_diagnostic(index, e),
        },
        ParseDefinitionError::Struct(e) => match &e.cause {
            ParseStructCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseStructCause::InvalidName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
            }
            ParseStructCause::Generics(e) => parse_generics_diagnostic(index, e),
            ParseStructCause::Fields(e) => parse_fields_diagnostic(index, e),
        },
        ParseDefinitionError::Enum(e) => match &e.cause {
            ParseEnumCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseEnumCause::InvalidName { at } | ParseEnumCause::InvalidVariantName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
            }
            ParseEnumCause::Generics(e) => parse_generics_diagnostic(index, e),
            ParseEnumCause::Field(e) => parse_fields_diagnostic(index, e),
            ParseEnumCause::Comment(e) => parse_comment_diagnostic(index, e),
            ParseEnumCause::Id(e) => parse_id_diagnostic(index, e),
        },
        ParseDefinitionError::Const(e) => match &e.cause {
            ParseConstCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseConstCause::UnexpectedChar { at, .. } | ParseConstCause::InvalidName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
            }
            ParseConstCause::Type(e) => parse_type_diagnostic(index, e),
            ParseConstCause::Literal(e) => match &e.cause {
                ParseLiteralCause::Parser(_, at) => {
                    Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
                }
                ParseLiteralCause::FoundReference { at }
                | ParseLiteralCause::InvalidInt { at }
                | ParseLiteralCause::ParseInt { at, .. } => {
                    Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
                }
            },
        },
        ParseDefinitionError::Alias(e) => match &e.cause {
            ParseAliasCause::Parser(_, at) => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
            }
            ParseAliasCause::InvalidName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
            }
            ParseAliasCause::Generics(e) => parse_generics_diagnostic(index, e),
            ParseAliasCause::Type(e) => parse_type_diagnostic(index, e),
        },
        ParseDefinitionError::Import(e) => parse_import_cause_diagnostic(index, &e.cause),
    }
}

fn parse_type_diagnostic(index: &Index, e: &ParseTypeError) -> Diagnostic {
    match &e.cause {
        ParseTypeCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseTypeCause::Type(e) => parse_type_diagnostic(index, e),
        ParseTypeCause::Segment(c) => parse_import_cause_diagnostic(index, c),
    }
}

fn parse_comment_diagnostic(index: &Index, e: &ParseCommentError) -> Diagnostic {
    Diagnostic::new_simple(get_range(index, e.at.clone()), e.to_string())
}

fn parse_import_cause_diagnostic(index: &Index, c: &ParseImportCause) -> Diagnostic {
    match c {
        ParseImportCause::Parser(_, at) | ParseImportCause::InvalidSegmentName { at } => {
            Diagnostic::new_simple(get_range(index, *at..*at), c.to_string())
        }
        ParseImportCause::StructName(c) => match c {
            ParseStructCause::Parser(_, at) | ParseStructCause::InvalidName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), c.to_string())
            }
            ParseStructCause::Generics(e) => parse_generics_diagnostic(index, e),
            ParseStructCause::Fields(e) => parse_fields_diagnostic(index, e),
        },
        ParseImportCause::EnumName(c) => match c {
            ParseEnumCause::Parser(_, at)
            | ParseEnumCause::InvalidName { at }
            | ParseEnumCause::InvalidVariantName { at } => {
                Diagnostic::new_simple(get_range(index, *at..*at), c.to_string())
            }
            ParseEnumCause::Generics(e) => parse_generics_diagnostic(index, e),
            ParseEnumCause::Field(e) => parse_fields_diagnostic(index, e),
            ParseEnumCause::Comment(e) => parse_comment_diagnostic(index, e),
            ParseEnumCause::Id(e) => parse_id_diagnostic(index, e),
        },
    }
}

fn parse_generics_diagnostic(index: &Index, e: &ParseGenericsError) -> Diagnostic {
    match &e.cause {
        mabo_parser::error::ParseGenericsCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        mabo_parser::error::ParseGenericsCause::InvalidName { at } => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
        }
    }
}

fn parse_fields_diagnostic(index: &Index, e: &ParseFieldsError) -> Diagnostic {
    match &e.cause {
        ParseFieldsCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseFieldsCause::InvalidName { at } => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
        }
        ParseFieldsCause::Type(e) => parse_type_diagnostic(index, e),
        ParseFieldsCause::Id(e) => parse_id_diagnostic(index, e),
        ParseFieldsCause::Comment(e) => parse_comment_diagnostic(index, e),
    }
}

fn parse_id_diagnostic(index: &Index, e: &ParseIdError) -> Diagnostic {
    Diagnostic::new_simple(get_range(index, e.at.clone()), e.to_string())
}

fn validate_schema_diagnostic(file: Url, index: &Index, e: validate::Error) -> Diagnostic {
    use validate::{DuplicateFieldId, DuplicateId, DuplicateName, Error, InvalidGenericType};

    let (message, first, second) = match e {
        Error::DuplicateId(e) => match e {
            DuplicateId::EnumVariant(e) => (e.to_string(), e.first, e.second),
            DuplicateId::Field(e) => match e {
                DuplicateFieldId::Named(e) => (e.to_string(), e.first, e.second),
                DuplicateFieldId::Unnamed(e) => (e.to_string(), e.first, e.second),
            },
        },
        Error::DuplicateName(e) => match e {
            DuplicateName::EnumVariant(e) => (e.to_string(), e.first, e.second),
            DuplicateName::Field(e) => (e.to_string(), e.first, e.second),
            DuplicateName::InModule(e) => (e.to_string(), e.first, e.second),
        },
        Error::InvalidGeneric(e) => match e {
            InvalidGenericType::Duplicate(e) => (e.to_string(), e.first, e.second),
            InvalidGenericType::Unused(e) => {
                let message = e.to_string();
                return Diagnostic::new_simple(get_range(index, e.declared), message);
            }
        },
        Error::TupleSize(e) => {
            let message = e.to_string();
            return Diagnostic::new_simple(get_range(index, e.declared), message);
        }
    };

    diagnostic_with_related(
        get_range(index, second),
        message,
        vec![lsp::DiagnosticRelatedInformation {
            location: lsp::Location::new(file, get_range(index, first)),
            message: "first used here".to_owned(),
        }],
    )
}

fn diagnostic_with_related(
    range: lsp::Range,
    message: String,
    related: Vec<lsp::DiagnosticRelatedInformation>,
) -> Diagnostic {
    Diagnostic::new(range, None, None, None, message, Some(related), None)
}

#[allow(clippy::expect_used)]
fn get_range(index: &Index, location: Range<usize>) -> lsp::Range {
    index
        .get_range(location)
        .expect("missing range information")
}
