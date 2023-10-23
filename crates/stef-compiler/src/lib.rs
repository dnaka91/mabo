pub use ids::{DuplicateFieldId, DuplicateId, DuplicateVariantId};
use stef_parser::{Definition, Schema};
use thiserror::Error;

use self::names::{DuplicateFieldName, DuplicateName};

mod ids;
mod names;

#[derive(Debug, Error)]
pub enum Error {
    #[error("duplicate ID found")]
    DuplicateId(#[from] DuplicateId),
    #[error("duplicate name found")]
    DuplicateName(#[from] DuplicateName),
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
    value.definitions.iter().try_for_each(validate_definition)
}

fn validate_definition(value: &Definition<'_>) -> Result<(), Error> {
    match value {
        Definition::Module(m) => {
            m.definitions.iter().try_for_each(validate_definition)?;
        }
        Definition::Struct(s) => {
            ids::validate_struct_ids(s)?;
            names::validate_struct_names(s)?;
        }
        Definition::Enum(e) => {
            ids::validate_enum_ids(e)?;
            names::validate_enum_names(e)?;
        }
        Definition::TypeAlias(_) | Definition::Const(_) | Definition::Import(_) => {}
    }

    Ok(())
}
