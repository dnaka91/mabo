use std::ops::Range;

use mabo_parser::{
    Const, DataType, Enum, Fields, Spanned, Struct, Type, TypeAlias, punctuated::ValuesIter,
};
use miette::{Diagnostic, diagnostic};
use thiserror::Error;

use crate::highlight;

/// Tuple with an invalid amount of elements was encountered.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "tuples with {} are invalid",
    highlight::focus(match amount {
        InvalidTupleAmount::Empty => "zero elements",
        InvalidTupleAmount::Single => "a single element",
        InvalidTupleAmount::TooLarge => "more than 12 elements"
    })
)]
#[diagnostic(help("a tuple must have between 2 and 12 elements"))]
pub struct TupleSize {
    /// The amount that's not allowed.
    pub amount: InvalidTupleAmount,
    /// Source location of the declaration.
    #[label("declared here")]
    pub declared: Range<usize>,
}

/// Possible amount of tuple elements that are invalid.
#[derive(Debug)]
pub enum InvalidTupleAmount {
    /// Tuple with zero elements.
    Empty,
    /// Single element tuple.
    Single,
    /// More than 12 elements.
    TooLarge,
}

pub(crate) fn validate_struct_tuples(value: &Struct<'_>) -> Result<(), TupleSize> {
    validate_field_tuples(&value.fields)
}

pub(crate) fn validate_enum_tuples(value: &Enum<'_>) -> Result<(), TupleSize> {
    value
        .variants
        .values()
        .try_for_each(|variant| validate_field_tuples(&variant.fields))
}

fn validate_field_tuples(value: &Fields<'_>) -> Result<(), TupleSize> {
    match value {
        Fields::Named(_, named) => named
            .values()
            .try_for_each(|field| validate_tuple_size(&field.ty)),
        Fields::Unnamed(_, unnamed) => unnamed
            .values()
            .try_for_each(|field| validate_tuple_size(&field.ty)),
        Fields::Unit => Ok(()),
    }
}

pub(crate) fn validate_alias_tuples(value: &TypeAlias<'_>) -> Result<(), TupleSize> {
    validate_tuple_size(&value.target)
}

pub(crate) fn validate_const_tuples(value: &Const<'_>) -> Result<(), TupleSize> {
    validate_tuple_size(&value.ty)
}

fn validate_tuple_size(value: &Type<'_>) -> Result<(), TupleSize> {
    visit_tuples(value, &mut |tuples| {
        let amount = match tuples.len() {
            0 => InvalidTupleAmount::Empty,
            1 => InvalidTupleAmount::Single,
            2..=12 => return Ok(()),
            _ => InvalidTupleAmount::TooLarge,
        };
        Err(TupleSize {
            amount,
            declared: value.span().into(),
        })
    })
}

/// Iterate recursively through the data type and invoke the closure on each discovered external
/// type.
fn visit_tuples<E>(
    value: &Type<'_>,
    visit: &mut impl FnMut(ValuesIter<'_, Type<'_>>) -> Result<(), E>,
) -> Result<(), E> {
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
        | DataType::NonZero { .. }
        | DataType::BoxString
        | DataType::BoxBytes => Ok(()),
        DataType::Vec { ty, .. }
        | DataType::HashSet { ty, .. }
        | DataType::Option { ty, .. }
        | DataType::Array { ty, .. } => visit_tuples(ty, visit),
        DataType::HashMap { key, value, .. } => {
            visit_tuples(key, visit)?;
            visit_tuples(value, visit)
        }
        DataType::Tuple { types, .. } => {
            visit(types.values())?;
            types.values().try_for_each(|ty| visit_tuples(ty, visit))
        }
        DataType::External(ty) => {
            if let Some(generics) = &ty.generics {
                generics
                    .values()
                    .try_for_each(|ty| visit_tuples(ty, visit))?;
            }
            Ok(())
        }
    }
}
