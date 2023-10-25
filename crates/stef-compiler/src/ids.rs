use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{Enum, Fields, Id, Spanned, Struct};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateId {
    #[error("duplicate ID in an enum variant")]
    #[diagnostic(transparent)]
    EnumVariant(#[from] DuplicateVariantId),
    #[error("duplicate ID in a field")]
    #[diagnostic(transparent)]
    Field(#[from] DuplicateFieldId),
}

#[derive(Debug, Diagnostic, Error)]
#[error("duplicate ID {} in enum variant `{name}`, already used in `{other_name}`", id.get())]
#[diagnostic(help("the IDs for each variant of an enum must be unique"))]
pub struct DuplicateVariantId {
    pub id: Id,
    pub name: String,
    pub other_name: String,
    #[label("first declared here")]
    pub first: Range<usize>,
    #[label("used here again")]
    pub second: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateFieldId {
    #[error("duplicate ID {} in field `{name}`, already used in `{other_name}`", id.get())]
    #[diagnostic(help("the IDs for each field must be unique"))]
    Named {
        id: Id,
        name: String,
        other_name: String,
        #[label("first declared here")]
        first: Range<usize>,
        #[label("used here again")]
        second: Range<usize>,
    },
    #[error("duplicate ID {} in position {position}, already used at {other_position}", id.get())]
    #[diagnostic(help("the IDs for each field must be unique"))]
    Unnamed {
        id: Id,
        position: usize,
        other_position: usize,
        #[label("first declared here")]
        first: Range<usize>,
        #[label("used here again")]
        second: Range<usize>,
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
                .insert(variant.id.get(), (variant.name.get(), variant.id.span()))
                .map(|(other_name, other_span)| {
                    DuplicateVariantId {
                        id: variant.id.clone(),
                        name: variant.name.get().to_owned(),
                        other_name: other_name.to_owned(),
                        first: other_span.into(),
                        second: variant.id.span().into(),
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
fn validate_field_ids(value: &Fields<'_>) -> Result<(), DuplicateFieldId> {
    match value {
        Fields::Named(named) => {
            let mut visited = HashMap::with_capacity(named.len());
            named
                .iter()
                .find_map(|field| {
                    visited
                        .insert(field.id.get(), (field.name.get(), field.id.span()))
                        .map(|(other_field, other_span)| DuplicateFieldId::Named {
                            id: field.id.clone(),
                            name: field.name.get().to_owned(),
                            other_name: other_field.to_owned(),
                            first: other_span.into(),
                            second: field.id.span().into(),
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
                    visited.insert(field.id.get(), (pos, field.id.span())).map(
                        |(other_position, other_span)| DuplicateFieldId::Unnamed {
                            id: field.id.clone(),
                            position: pos + 1,
                            other_position: other_position + 1,
                            first: other_span.into(),
                            second: field.id.span().into(),
                        },
                    )
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unit => {}
    }

    Ok(())
}
