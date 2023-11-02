#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::{
    error::Error,
    fmt::{self, Display},
};

pub use ids::{DuplicateFieldId, DuplicateId, DuplicateVariantId};
use miette::{Diagnostic, NamedSource};
use stef_parser::{Definition, Schema};
use thiserror::Error;

use self::{
    generics::InvalidGenericType,
    names::{DuplicateFieldName, DuplicateName},
    resolve::ResolveError,
};

mod generics;
mod highlight;
mod ids;
mod names;
mod resolve;

#[derive(Debug, Diagnostic, Error)]
pub enum ValidationError {
    #[error("duplicate ID found")]
    #[diagnostic(transparent)]
    DuplicateId(#[from] DuplicateId),
    #[error("duplicate name found")]
    #[diagnostic(transparent)]
    DuplicateName(#[from] DuplicateName),
    #[error("invalid generic type found")]
    #[diagnostic(transparent)]
    InvalidGeneric(#[from] InvalidGenericType),
}

impl From<DuplicateFieldId> for ValidationError {
    fn from(v: DuplicateFieldId) -> Self {
        Self::DuplicateId(v.into())
    }
}

impl From<DuplicateFieldName> for ValidationError {
    fn from(v: DuplicateFieldName) -> Self {
        Self::DuplicateName(v.into())
    }
}

pub fn validate_schema(value: &Schema<'_>) -> Result<(), ValidationError> {
    names::validate_names_in_module(&value.definitions)?;
    value.definitions.iter().try_for_each(validate_definition)
}

fn validate_definition(value: &Definition<'_>) -> Result<(), ValidationError> {
    match value {
        Definition::Module(m) => {
            names::validate_names_in_module(&m.definitions)?;
            m.definitions.iter().try_for_each(validate_definition)?;
        }
        Definition::Struct(s) => {
            ids::validate_struct_ids(s)?;
            names::validate_struct_names(s)?;
            generics::validate_struct_generics(s)?;
        }
        Definition::Enum(e) => {
            ids::validate_enum_ids(e)?;
            names::validate_enum_names(e)?;
            generics::validate_enum_generics(e)?;
        }
        Definition::TypeAlias(_) | Definition::Const(_) | Definition::Import(_) => {}
    }

    Ok(())
}

#[derive(Debug)]
pub struct ResolutionError {
    source_code: NamedSource,
    cause: resolve::ResolveError,
}

impl Error for ResolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

impl Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("type resolution failed")
    }
}

impl Diagnostic for ResolutionError {
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

pub fn resolve_schemas(values: &[(&str, &Schema<'_>)]) -> Result<(), ResolutionError> {
    let modules = values
        .iter()
        .map(|(name, schema)| (*name, resolve::resolve_types(name, schema)))
        .collect::<Vec<_>>();

    for (schema, module) in modules
        .iter()
        .enumerate()
        .map(|(i, (_, module))| (values[i].1, module))
    {
        let mut missing = Vec::new();
        resolve::resolve_module_types(module, &mut missing);

        let imports =
            resolve::resolve_module_imports(module, &modules).map_err(|e| ResolutionError {
                source_code: NamedSource::new(
                    schema
                        .path
                        .as_ref()
                        .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                    schema.source.to_owned(),
                ),
                cause: ResolveError::Import(e),
            })?;

        for ty in missing {
            resolve::resolve_type_remotely(ty, &imports).map_err(|e| ResolutionError {
                source_code: NamedSource::new(
                    schema
                        .path
                        .as_ref()
                        .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                    schema.source.to_owned(),
                ),
                cause: e,
            })?;
        }
    }

    Ok(())
}
