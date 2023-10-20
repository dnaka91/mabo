use std::collections::HashMap;

use stef_parser::{Enum, Fields, Id, Struct};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DuplicateId {
    #[error("duplicate ID in an enum variant")]
    EnumVariant(#[from] DuplicateVariantId),
    #[error("duplicate ID in a field")]
    Field(#[from] DuplicateFieldId),
}

#[derive(Debug, Error)]
#[error("duplicate ID {} in enum variant `{name}`, already used in `{other_name}`", id.0)]
pub struct DuplicateVariantId {
    pub id: Id,
    pub name: String,
    pub other_name: String,
}

#[derive(Debug, Error)]
pub enum DuplicateFieldId {
    #[error("duplicate ID {} in field `{name}`, already used in `{other_name}`", id.0)]
    Named {
        id: Id,
        name: String,
        other_name: String,
    },
    #[error("duplicate ID {} in field {position}, already used in {other_position}", id.0)]
    Unnamed {
        id: Id,
        position: usize,
        other_position: usize,
    },
}

/// Ensure all IDs inside a struct are unique (which are the field IDs).
pub(crate) fn validate_struct_ids(value: &Struct<'_>) -> Result<(), DuplicateFieldId> {
    validate_field_ids(&value.fields)
}

/// Ensure all IDs inside an enum are unique, which means all variants have a unique ID, plus all
/// potential fields in a variant are unique (within that variant).
pub(crate) fn validate_enum_ids(value: &Enum<'_>) -> Result<(), DuplicateId> {
    let mut visited = HashMap::with_capacity(value.variants.len());
    value
        .variants
        .iter()
        .find_map(|variant| {
            visited
                .insert(variant.id, variant.name)
                .map(|other_name| {
                    DuplicateVariantId {
                        id: variant.id,
                        name: variant.name.to_owned(),
                        other_name: other_name.to_owned(),
                    }
                    .into()
                })
                .or_else(|| {
                    validate_field_ids(&variant.fields)
                        .err()
                        .map(DuplicateId::from)
                })
        })
        .map_or(Ok(()), Err)
}

/// Ensure all field IDs of a struct or enum are unique.
fn validate_field_ids(value: &Fields) -> Result<(), DuplicateFieldId> {
    match value {
        Fields::Named(named) => {
            let mut visited = HashMap::with_capacity(named.len());
            named
                .iter()
                .find_map(|field| {
                    visited.insert(field.id, field.name).map(|other_field| {
                        DuplicateFieldId::Named {
                            id: field.id,
                            name: field.name.to_owned(),
                            other_name: other_field.to_owned(),
                        }
                    })
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unnamed(unnamed) => {
            let mut visited = HashMap::with_capacity(unnamed.len());
            unnamed
                .iter()
                .enumerate()
                .find_map(|(pos, field)| {
                    visited
                        .insert(field.id, pos)
                        .map(|other_position| DuplicateFieldId::Unnamed {
                            id: field.id,
                            position: pos,
                            other_position,
                        })
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unit => {}
    }

    Ok(())
}
