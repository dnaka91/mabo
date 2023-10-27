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
};

mod generics;
mod ids;
mod names;

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
