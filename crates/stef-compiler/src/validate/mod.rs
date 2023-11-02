//! Ensure several conditions for a single schema are met, which are difficult to verify during the
//! parsing step.

use miette::Diagnostic;
use stef_parser::{Definition, Schema};
use thiserror::Error;

pub use self::{
    generics::{DuplicateGenericName, InvalidGenericType, UnusedGeneric},
    ids::{
        DuplicateFieldId, DuplicateId, DuplicateNamedFieldId, DuplicateUnnamedFieldId,
        DuplicateVariantId,
    },
    names::{DuplicateFieldName, DuplicateName, DuplicateNameInModule, DuplicateVariantName},
};

mod generics;
mod ids;
mod names;

/// Reason why a schema was invalid.
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    /// Duplicate ID was used in a definition.
    #[error("duplicate ID found")]
    #[diagnostic(transparent)]
    DuplicateId(#[from] DuplicateId),
    /// Duplicate name was used in a definition, or its name clashes with another one.
    #[error("duplicate name found")]
    #[diagnostic(transparent)]
    DuplicateName(#[from] DuplicateName),
    /// Generic type parameters are invalid.
    #[error("invalid generic type found")]
    #[diagnostic(transparent)]
    InvalidGeneric(#[from] InvalidGenericType),
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

/// Ensure the schema doesn't include invalid definitions, which would be difficult to validate
/// during the parsing step.
///
/// Currently, it checks that:
/// - All definitions (struct, enums, modules, ...) have a unique name within their module
///   namespace.
/// - IDs in field names or enum variant names are unique.
/// - Fields names in structs or enum variants are unique.
/// - Generic type parameters in a struct or enum are unique.
/// - All generic type parameters are used.
pub fn schema(value: &Schema<'_>) -> Result<(), Error> {
    names::validate_names_in_module(&value.definitions)?;
    value.definitions.iter().try_for_each(definition)
}

fn definition(value: &Definition<'_>) -> Result<(), Error> {
    match value {
        Definition::Module(m) => {
            names::validate_names_in_module(&m.definitions)?;
            m.definitions.iter().try_for_each(definition)?;
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
