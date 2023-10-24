use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{Enum, Fields, Spanned, Struct};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateName {
    #[error("duplicate name in an enum variant")]
    #[diagnostic(transparent)]
    EnumVariant(#[from] DuplicateVariantName),
    #[error("duplicate name in a field")]
    #[diagnostic(transparent)]
    Field(#[from] DuplicateFieldName),
}

#[derive(Debug, Diagnostic, Error)]
#[error("duplicate variant name `{name}` in enum")]
#[diagnostic(help("the names of each variant must be unique"))]
pub struct DuplicateVariantName {
    pub name: String,
    #[label("first declared here")]
    pub first: Range<usize>,
    #[label("used here again")]
    pub second: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
#[error("duplicate field name `{name}`")]
#[diagnostic(help("the names of each field must be unique"))]
pub struct DuplicateFieldName {
    pub name: String,
    #[label("first declared here")]
    pub first: Range<usize>,
    #[label("used here again")]
    pub second: Range<usize>,
}

/// Ensure all field names inside a struct are unique.
pub(crate) fn validate_struct_names(value: &Struct<'_>) -> Result<(), DuplicateFieldName> {
    validate_field_names(&value.fields)
}

/// Ensure all names inside an enum are unique, which means all variants have a unique name, plus
/// all potential fields in a variant are unique (within that variant).
pub(crate) fn validate_enum_names(value: &Enum<'_>) -> Result<(), DuplicateName> {
    let mut visited = HashMap::with_capacity(value.variants.len());
    value
        .variants
        .iter()
        .find_map(|variant| {
            visited
                .insert(variant.name.get(), variant.name.span())
                .map(|first| {
                    DuplicateVariantName {
                        name: variant.name.get().to_owned(),
                        first: first.into(),
                        second: variant.name.span().into(),
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
fn validate_field_names(value: &Fields<'_>) -> Result<(), DuplicateFieldName> {
    match value {
        Fields::Named(named) => {
            let mut visited = HashMap::with_capacity(named.len());
            named
                .iter()
                .find_map(|field| {
                    visited
                        .insert(field.name.get(), field.name.span())
                        .map(|first| DuplicateFieldName {
                            name: field.name.get().to_owned(),
                            first: first.into(),
                            second: field.name.span().into(),
                        })
                })
                .map_or(Ok(()), Err)?;
        }
        Fields::Unnamed(_) | Fields::Unit => {}
    }

    Ok(())
}
