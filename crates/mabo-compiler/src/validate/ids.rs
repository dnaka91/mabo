use std::ops::Range;

use mabo_parser::{Enum, Fields, Id, Spanned, Struct};
use miette::Diagnostic;
use rustc_hash::{FxBuildHasher, FxHashMap};
use thiserror::Error;

use crate::IdGenerator;

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
    /// Source location of the first occurrence.
    #[label("first declared here")]
    pub first: Range<usize>,
    /// Source location of the duplicate.
    #[label("used here again")]
    pub second: Range<usize>,
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
    /// Source location of the first occurrence.
    #[label("first declared here")]
    pub first: Range<usize>,
    /// Source location of the duplicate.
    #[label("used here again")]
    pub second: Range<usize>,
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
    /// Source location of the first occurrence.
    #[label("first declared here")]
    pub first: Range<usize>,
    /// Source location of the duplicate.
    #[label("used here again")]
    pub second: Range<usize>,
}

/// Ensure all IDs inside a struct are unique (which are the field IDs).
pub(crate) fn validate_struct_ids(value: &Struct<'_>) -> Result<(), DuplicateFieldId> {
    validate_field_ids(&value.fields)
}

/// Ensure all IDs inside an enum are unique, which means all variants have a unique ID, plus all
/// potential fields in a variant are unique (within that variant).
pub(crate) fn validate_enum_ids(value: &Enum<'_>) -> Result<(), DuplicateId> {
    let mut visited = FxHashMap::with_capacity_and_hasher(value.variants.len(), FxBuildHasher);
    let mut id_gen = IdGenerator::new();

    value
        .variants
        .values()
        .find_map(|variant| {
            let id = id_gen.next_with_span(variant.id.as_ref(), || variant.span());

            visited
                .insert(id.get(), (variant.name.get(), id.span()))
                .map(|(other_name, other_span)| {
                    DuplicateVariantId {
                        name: variant.name.get().to_owned(),
                        other_name: other_name.to_owned(),
                        first: other_span.into(),
                        second: id.span().into(),
                        id,
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
        Fields::Named(_, named) => {
            let mut visited = FxHashMap::with_capacity_and_hasher(named.len(), FxBuildHasher);
            let mut id_gen = IdGenerator::new();

            named
                .values()
                .find_map(|field| {
                    let id = id_gen.next_with_span(field.id.as_ref(), || field.span());

                    visited.insert(id.get(), (field.name.get(), id.span())).map(
                        |(other_field, other_span)| DuplicateNamedFieldId {
                            name: field.name.get().to_owned(),
                            other_name: other_field.to_owned(),
                            first: other_span.into(),
                            second: id.span().into(),
                            id,
                        },
                    )
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unnamed(_, unnamed) => {
            let mut visited = FxHashMap::with_capacity_and_hasher(unnamed.len(), FxBuildHasher);
            let mut id_gen = IdGenerator::new();

            unnamed
                .values()
                .enumerate()
                .find_map(|(pos, field)| {
                    let id = id_gen.next_with_span(field.id.as_ref(), || field.span());

                    visited.insert(id.get(), (pos, id.span())).map(
                        |(other_position, other_span)| DuplicateUnnamedFieldId {
                            position: pos + 1,
                            other_position: other_position + 1,
                            first: other_span.into(),
                            second: id.span().into(),
                            id,
                        },
                    )
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unit => {}
    }

    Ok(())
}
