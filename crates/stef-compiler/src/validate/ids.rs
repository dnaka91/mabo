use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{Enum, Fields, Id, Spanned, Struct};
use thiserror::Error;

/// Duplicate ID was encountered for two elements in the same scope.
#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateId {
    /// Two enum variants use the same ID.
    #[error("duplicate ID in an enum variant")]
    #[diagnostic(transparent)]
    EnumVariant(#[from] DuplicateVariantId),
    /// Two fields use the same ID.
    #[error("duplicate ID in a field")]
    #[diagnostic(transparent)]
    Field(#[from] DuplicateFieldId),
}

/// Duplicate ID for enum variants.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate ID {} in enum variant `{name}`, already used in `{other_name}`", id.get())]
#[diagnostic(help("the IDs for each variant of an enum must be unique"))]
pub struct DuplicateVariantId {
    /// The duplicate ID.
    pub id: Id,
    /// Name of the variant that tries to use the same ID again.
    pub name: String,
    /// Name of the variant that used the ID for the first time.
    pub other_name: String,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
}

/// Duplicate ID for fields of a struct or enum variant.
#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateFieldId {
    /// Found duplicate IDs in named fields.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Named(#[from] DuplicateNamedFieldId),
    /// Found duplicate IDs in **un**named fields.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Unnamed(#[from] DuplicateUnnamedFieldId),
}

/// Duplicate ID for named fields.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate ID {} in field `{name}`, already used in `{other_name}`", id.get())]
#[diagnostic(help("the IDs for each field must be unique"))]
pub struct DuplicateNamedFieldId {
    /// The duplicate ID.
    pub id: Id,
    /// Name of the field that tries to use the same ID again.
    pub name: String,
    /// Name of the field that used the ID for the first time.
    pub other_name: String,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
}

/// Duplicate ID for unnamed fields.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate ID {} in position {position}, already used at {other_position}", id.get())]
#[diagnostic(help("the IDs for each field must be unique"))]
pub struct DuplicateUnnamedFieldId {
    /// The duplicate ID.
    pub id: Id,
    /// 1-based position of the field that tries to use the same ID again.
    pub position: usize,
    /// 1-base position of the field that used the ID for the first time.
    pub other_position: usize,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
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
                        .map(|(other_field, other_span)| DuplicateNamedFieldId {
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
                        |(other_position, other_span)| DuplicateUnnamedFieldId {
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
