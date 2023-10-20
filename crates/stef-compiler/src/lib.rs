pub use ids::{DuplicateFieldId, DuplicateId, DuplicateVariantId};
use stef_parser::{Definition, Schema};
use thiserror::Error;

mod ids;

#[derive(Debug, Error)]
pub enum Error {
    #[error("duplicate ID found")]
    DuplicateId(#[from] DuplicateId),
}

impl From<DuplicateFieldId> for Error {
    fn from(v: DuplicateFieldId) -> Self {
        Self::DuplicateId(v.into())
    }
}

pub fn validate_schema(value: &Schema<'_>) -> Result<(), Error> {
    value.definitions.iter().try_for_each(validate_definition)
}

fn validate_definition(value: &Definition<'_>) -> Result<(), Error> {
    match value {
        Definition::Module(m) => m.definitions.iter().try_for_each(validate_definition),
        Definition::Struct(s) => ids::validate_struct_ids(s).map_err(Into::into),
        Definition::Enum(e) => ids::validate_enum_ids(e).map_err(Into::into),
        Definition::TypeAlias(_) | Definition::Const(_) | Definition::Import(_) => Ok(()),
    }
}
