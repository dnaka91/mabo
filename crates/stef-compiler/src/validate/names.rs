use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{Definition, Enum, Fields, Import, Spanned, Struct};
use thiserror::Error;

/// Duplicate name was encountered for two elements in the same scope.
#[derive(Debug, Diagnostic, Error)]
pub enum DuplicateName {
    /// Two variants of an enum have the same name.
    #[error("duplicate name in an enum variant")]
    #[diagnostic(transparent)]
    EnumVariant(#[from] DuplicateVariantName),
    /// Two fields in a struct or enum variant have the same name.
    #[error("duplicate name in a field")]
    #[diagnostic(transparent)]
    Field(#[from] DuplicateFieldName),
    /// Two definitions in a module have the same name.
    #[error("duplicate name in the scope of a module")]
    #[diagnostic(transparent)]
    InModule(#[from] DuplicateNameInModule),
}

/// Duplicate name for enum variants.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate variant name `{name}` in enum")]
#[diagnostic(help("the names of each variant must be unique"))]
pub struct DuplicateVariantName {
    /// Name of the variant.
    pub name: String,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
}

/// Duplicate name for fields of a struct or enum variant.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate field name `{name}`")]
#[diagnostic(help("the names of each field must be unique"))]
pub struct DuplicateFieldName {
    /// Name of the field.
    pub name: String,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
}

/// Duplicate name for definitions inside a module.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate definition name `{name}`")]
#[diagnostic(help(
    "the names of each definition must be unique and not collide with other declarations"
))]
pub struct DuplicateNameInModule {
    /// Name of the declaration.
    pub name: String,
    #[label("first declared here")]
    first: Range<usize>,
    #[label("used here again")]
    second: Range<usize>,
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

pub(crate) fn validate_names_in_module(value: &[Definition<'_>]) -> Result<(), DuplicateName> {
    let mut visited = HashMap::with_capacity(value.len());
    value
        .iter()
        .find_map(|definition| {
            let name = match definition {
                Definition::Module(m) => &m.name,
                Definition::Struct(s) => &s.name,
                Definition::Enum(e) => &e.name,
                Definition::TypeAlias(a) => &a.name,
                Definition::Const(c) => &c.name,
                Definition::Import(Import {
                    element: Some(name),
                    ..
                }) => name,
                Definition::Import(Import { segments, .. }) => segments.last()?,
            };
            visited.insert(name.get(), name.span()).map(|first| {
                DuplicateNameInModule {
                    name: name.get().to_owned(),
                    first: first.into(),
                    second: name.span().into(),
                }
                .into()
            })
        })
        .map_or(Ok(()), Err)
}
