use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use mabo_parser::{DataType, Enum, ExternalType, Fields, Generics, Span, Spanned, Struct, Type};
use thiserror::Error;

/// Generic type parameters are considered invalid.
#[derive(Debug, Diagnostic, Error)]
pub enum InvalidGenericType {
    /// Two parameters with the same name found.
    #[error("duplicate generic type name found")]
    #[diagnostic(transparent)]
    Duplicate(#[from] DuplicateGenericName),
    /// Unused parameter found.
    #[error("unused generic type argument found")]
    #[diagnostic(transparent)]
    Unused(#[from] UnusedGeneric),
}

/// Duplicate name for type parameters.
#[derive(Debug, Diagnostic, Error)]
#[error("duplicate generic type name `{name}`")]
#[diagnostic(help("the names of each generic type must be unique"))]
pub struct DuplicateGenericName {
    /// Name of the parameter.
    pub name: String,
    /// Source location of the first occurrence.
    #[label("first declared here")]
    pub first: Range<usize>,
    /// Source location of the duplicate.
    #[label("used here again")]
    pub second: Range<usize>,
}

/// Defined but unused type parameter.
#[derive(Debug, Diagnostic, Error)]
#[error("unused generic type argument `{name}`")]
#[diagnostic(help("each declared generic must be used in some way"))]
pub struct UnusedGeneric {
    /// Name of the parameter.
    pub name: String,
    /// Source location of the declaration.
    #[label("declared here")]
    pub declared: Range<usize>,
}

/// Ensure all generics in a struct are unique and used.
pub fn validate_struct_generics(value: &Struct<'_>) -> Result<(), InvalidGenericType> {
    validate_duplicate_generics(&value.generics)?;

    let mut unvisited = value
        .generics
        .0
        .iter()
        .map(|gen| (gen.get(), gen.span()))
        .collect::<HashMap<_, _>>();

    validate_field_generics(&value.fields, &mut unvisited);

    unvisited.into_iter().next().map_or(Ok(()), |(name, span)| {
        Err(UnusedGeneric {
            name: name.to_owned(),
            declared: span.into(),
        }
        .into())
    })
}

/// Ensure all generics in an enum are unique and used.
pub fn validate_enum_generics(value: &Enum<'_>) -> Result<(), InvalidGenericType> {
    validate_duplicate_generics(&value.generics)?;

    let mut unvisited = value
        .generics
        .0
        .iter()
        .map(|gen| (gen.get(), gen.span()))
        .collect::<HashMap<_, _>>();

    for variant in &value.variants {
        validate_field_generics(&variant.fields, &mut unvisited);
    }

    unvisited.into_iter().next().map_or(Ok(()), |(name, span)| {
        Err(UnusedGeneric {
            name: name.to_owned(),
            declared: span.into(),
        }
        .into())
    })
}

/// Ensure all generic type arguments are unique within a struct or enum.
fn validate_duplicate_generics(value: &Generics<'_>) -> Result<(), DuplicateGenericName> {
    let mut visited = HashMap::with_capacity(value.0.len());
    value
        .0
        .iter()
        .find_map(|name| {
            visited
                .insert(name.get(), name.span())
                .map(|first| DuplicateGenericName {
                    name: name.get().to_owned(),
                    first: first.into(),
                    second: name.span().into(),
                })
        })
        .map_or(Ok(()), Err)
}

/// Iterate over all the fields and mark any generic types as used when disvored as type for a
/// field.
fn validate_field_generics(value: &Fields<'_>, unvisited: &mut HashMap<&str, Span>) {
    match &value {
        Fields::Named(named) => {
            for field in named {
                visit_externals(&field.ty, &mut |external| {
                    if external.path.is_empty() && external.generics.is_empty() {
                        unvisited.remove(external.name.get());
                    }
                });
            }
        }
        Fields::Unnamed(unnamed) => {
            for field in unnamed {
                visit_externals(&field.ty, &mut |external| {
                    if external.path.is_empty() && external.generics.is_empty() {
                        unvisited.remove(external.name.get());
                    }
                });
            }
        }
        Fields::Unit => {}
    }
}

/// Iterate recursively through the data type and invoke the closure on each discovered external
/// type.
fn visit_externals(value: &Type<'_>, visit: &mut impl FnMut(&ExternalType<'_>)) {
    match &value.value {
        DataType::Bool
        | DataType::U8
        | DataType::U16
        | DataType::U32
        | DataType::U64
        | DataType::U128
        | DataType::I8
        | DataType::I16
        | DataType::I32
        | DataType::I64
        | DataType::I128
        | DataType::F32
        | DataType::F64
        | DataType::String
        | DataType::StringRef
        | DataType::Bytes
        | DataType::BytesRef
        | DataType::NonZero(_)
        | DataType::BoxString
        | DataType::BoxBytes => {}
        DataType::Vec(ty)
        | DataType::HashSet(ty)
        | DataType::Option(ty)
        | DataType::Array(ty, _) => visit_externals(ty, visit),
        DataType::HashMap(kv) => {
            visit_externals(&kv.0, visit);
            visit_externals(&kv.1, visit);
        }
        DataType::Tuple(types) => {
            for ty in types {
                visit_externals(ty, visit);
            }
        }
        DataType::External(ty) => {
            visit(ty);

            for ty in &ty.generics {
                visit_externals(ty, visit);
            }
        }
    }
}
