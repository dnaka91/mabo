use std::ops::Range;

use line_index::{LineIndex, TextSize, WideEncoding};
use lsp_types::{self as lsp, Diagnostic, Url};
use stef_compiler::validate;
use stef_parser::{
    error::{
        ParseAliasCause, ParseAttributeCause, ParseCommentError, ParseConstCause,
        ParseDefinitionError, ParseEnumCause, ParseFieldsCause, ParseFieldsError,
        ParseGenericsError, ParseIdError, ParseImportCause, ParseLiteralCause, ParseModuleCause,
        ParseSchemaCause, ParseSchemaError, ParseStructCause, ParseTypeCause, ParseTypeError,
    },
    Schema,
};

pub fn compile(file: Url, schema: &str) -> std::result::Result<Schema<'_>, Diagnostic> {
    let index = LineIndex::new(schema);

    let parsed = stef_parser::Schema::parse(schema, None)
        .map_err(|e| parse_schema_diagnostic(&index, &e))?;

    stef_compiler::validate_schema(&parsed)
        .map_err(|e| validate_schema_diagnostic(file, &index, e))?;

    Ok(parsed)
}

fn parse_schema_diagnostic(index: &LineIndex, e: &ParseSchemaError) -> Diagnostic {
    match &e.cause {
        ParseSchemaCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseSchemaCause::Definition(e) => parse_definition_diagnostic(index, e),
    }
}

fn parse_definition_diagnostic(index: &LineIndex, e: &ParseDefinitionError) -> Diagnostic {
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

fn parse_type_diagnostic(index: &LineIndex, e: &ParseTypeError) -> Diagnostic {
    match &e.cause {
        ParseTypeCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        ParseTypeCause::Type(e) => parse_type_diagnostic(index, e),
        ParseTypeCause::Segment(c) => parse_import_cause_diagnostic(index, c),
    }
}

fn parse_comment_diagnostic(index: &LineIndex, e: &ParseCommentError) -> Diagnostic {
    Diagnostic::new_simple(get_range(index, e.at.clone()), e.to_string())
}

fn parse_import_cause_diagnostic(index: &LineIndex, c: &ParseImportCause) -> Diagnostic {
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

fn parse_generics_diagnostic(index: &LineIndex, e: &ParseGenericsError) -> Diagnostic {
    match &e.cause {
        stef_parser::error::ParseGenericsCause::Parser(_, at) => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.to_string())
        }
        stef_parser::error::ParseGenericsCause::InvalidName { at } => {
            Diagnostic::new_simple(get_range(index, *at..*at), e.cause.to_string())
        }
    }
}

fn parse_fields_diagnostic(index: &LineIndex, e: &ParseFieldsError) -> Diagnostic {
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

fn parse_id_diagnostic(index: &LineIndex, e: &ParseIdError) -> Diagnostic {
    Diagnostic::new_simple(get_range(index, e.at.clone()), e.to_string())
}

fn validate_schema_diagnostic(file: Url, index: &LineIndex, e: validate::Error) -> Diagnostic {
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

#[allow(clippy::cast_possible_truncation, clippy::expect_used)]
fn get_range(index: &LineIndex, location: Range<usize>) -> lsp::Range {
    let start = index
        .to_wide(
            WideEncoding::Utf16,
            index.line_col(TextSize::new(location.start as u32)),
        )
        .expect("missing utf-16 start position");

    let end = index
        .to_wide(
            WideEncoding::Utf16,
            index.line_col(TextSize::new(location.end as u32)),
        )
        .expect("missing utf-16 end position");

    lsp::Range::new(
        lsp::Position::new(start.line, start.col),
        lsp::Position::new(end.line, end.col),
    )
}
