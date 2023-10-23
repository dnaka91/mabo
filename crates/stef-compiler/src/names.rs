use std::collections::HashSet;

use stef_parser::{Enum, Fields, Struct};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DuplicateName {
    #[error("duplicate name in an enum variant")]
    EnumVariant(#[from] DuplicateVariantName),
    #[error("duplicate name in a field")]
    Field(#[from] DuplicateFieldName),
}

#[derive(Debug, Error)]
#[error("duplicate variant name `{name}` in enum")]
pub struct DuplicateVariantName {
    pub name: String,
}

#[derive(Debug, Error)]
#[error("duplicate field name `{name}`")]
pub struct DuplicateFieldName {
    pub name: String,
}

/// Ensure all field names inside a struct are unique.
pub(crate) fn validate_struct_names(value: &Struct<'_>) -> Result<(), DuplicateFieldName> {
    validate_field_names(&value.fields)
}

/// Ensure all names inside an enum are unique, which means all variants have a unique name, plus
/// all potential fields in a variant are unique (within that variant).
pub(crate) fn validate_enum_names(value: &Enum<'_>) -> Result<(), DuplicateName> {
    let mut visited = HashSet::with_capacity(value.variants.len());
    value
        .variants
        .iter()
        .find_map(|variant| {
            (!visited.insert(variant.name))
                .then(|| {
                    DuplicateVariantName {
                        name: variant.name.to_owned(),
                    }
                    .into()
                })
                .or_else(|| {
                    validate_field_names(&variant.fields)
                        .err()
                        .map(DuplicateName::from)
                })
        })
        .map_or(Ok(()), Err)
}

/// Ensure all field names of a struct or enum are unique.
fn validate_field_names(value: &Fields) -> Result<(), DuplicateFieldName> {
    match value {
        Fields::Named(named) => {
            let mut visited = HashSet::with_capacity(named.len());
            named
                .iter()
                .find_map(|field| {
                    (!visited.insert(field.name)).then(|| DuplicateFieldName {
                        name: field.name.to_owned(),
                    })
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unnamed(_) | Fields::Unit => {}
    }

    Ok(())
}
