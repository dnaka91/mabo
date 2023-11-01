#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

pub use ids::{DuplicateFieldId, DuplicateId, DuplicateVariantId};
use miette::Diagnostic;
use stef_parser::{Definition, Schema};
use thiserror::Error;

use self::{
    generics::InvalidGenericType,
    names::{DuplicateFieldName, DuplicateName},
    resolve::{ResolveImport, ResolveLocal, ResolveRemote},
};

mod generics;
mod highlight;
mod ids;
mod names;
mod resolve;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error("duplicate ID found")]
    #[diagnostic(transparent)]
    DuplicateId(#[from] DuplicateId),
    #[error("duplicate name found")]
    #[diagnostic(transparent)]
    DuplicateName(#[from] DuplicateName),
    #[error("invalid generic type found")]
    #[diagnostic(transparent)]
    InvalidGeneric(#[from] InvalidGenericType),
    #[error("type resolution failed")]
    #[diagnostic(transparent)]
    Resolve(#[from] resolve::ResolveError),
}

impl From<DuplicateFieldId> for Error {
    fn from(v: DuplicateFieldId) -> Self {
        Self::DuplicateId(v.into())
    }
}

impl From<DuplicateFieldName> for Error {
    fn from(v: DuplicateFieldName) -> Self {
        Self::DuplicateName(v.into())
    }
}

impl From<ResolveLocal> for Error {
    fn from(v: ResolveLocal) -> Self {
        Self::Resolve(v.into())
    }
}

impl From<ResolveImport> for Error {
    fn from(v: ResolveImport) -> Self {
        Self::Resolve(v.into())
    }
}

impl From<ResolveRemote> for Error {
    fn from(v: ResolveRemote) -> Self {
        Self::Resolve(v.into())
    }
}

pub fn validate_schema(value: &Schema<'_>) -> Result<(), Error> {
    names::validate_names_in_module(&value.definitions)?;
    value.definitions.iter().try_for_each(validate_definition)
}

fn validate_definition(value: &Definition<'_>) -> Result<(), Error> {
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

pub fn resolve_schemas(values: &[(&str, &Schema<'_>)]) -> Result<(), Error> {
    let modules = values
        .iter()
        .map(|(name, value)| (*name, resolve::resolve_types(name, value)))
        .collect::<Vec<_>>();

    for (_, module) in &modules {
        let mut missing = Vec::new();
        resolve::resolve_module_types(module, &mut missing);

        let imports = resolve::resolve_module_imports(module, &modules)?;

        for ty in missing {
            resolve::resolve_type_remotely(ty, &imports)?;
        }
    }

    Ok(())
}
