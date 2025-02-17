//! Collection of utilities that can retrieve metadata information about a schema.
//!
//! Several tools like the LSP server and docs generator can use this common logic to provide
//! additional information about the schema shown.

use std::{borrow::Cow, fmt::Write};

use mabo_compiler::simplify::{FieldKind, Fields, Type, Variant};

/// Get the next free ID for an enum variant.
#[must_use]
pub fn next_variant_id(variants: &[Variant<'_>]) -> u32 {
    variants.iter().map(|variant| variant.id).max().unwrap_or(0) + 1
}

/// Get the next free ID for a struct or enum variant field.
#[must_use]
pub fn next_field_id(fields: &Fields<'_>) -> Option<u32> {
    if fields.kind == FieldKind::Unit {
        None
    } else {
        Some(
            fields
                .fields
                .iter()
                .map(|field| field.id)
                .max()
                .unwrap_or(0)
                + 1,
        )
    }
}

/// Information about the wire (encoded) size of a data type.
pub struct WireSize {
    label: Cow<'static, str>,
    min: usize,
    max: Option<usize>,
    inner: Vec<(Cow<'static, str>, Option<WireSize>)>,
}

impl WireSize {
    fn fixed(label: impl Into<Cow<'static, str>>, size: usize) -> Self {
        Self {
            label: label.into(),
            min: size,
            max: Some(size),
            inner: Vec::new(),
        }
    }

    fn range(label: impl Into<Cow<'static, str>>, min: usize, max: usize) -> Self {
        Self {
            label: label.into(),
            min,
            max: Some(max),
            inner: Vec::new(),
        }
    }

    fn min(label: impl Into<Cow<'static, str>>, min: usize) -> Self {
        Self {
            label: label.into(),
            min,
            max: None,
            inner: Vec::new(),
        }
    }

    /// Write the information in a descriptive tree structure made out of Markdown lists.
    pub fn print(&self, buf: &mut String, indent: usize) {
        let _ = write!(buf, "**{}** ", self.label);

        let _ = match self.max {
            Some(max) if self.min == max => write!(buf, "`{max}`"),
            Some(max) => write!(buf, "`{}..{max}`", self.min),
            None => write!(buf, "`{}..`", self.min),
        };

        for (label, size) in &self.inner {
            let _ = write!(buf, "\n{:indent$}- {label}: ", "", indent = indent + 2);
            if let Some(size) = size {
                size.print(buf, indent + 2);
            } else {
                let _ = write!(buf, "_unknown_");
            }
        }
    }
}

/// Calculate the expected encoded byte size for a type.
///
/// The resulting size can have various forms of precision, depending on the data type. For example,
/// it could be of fixed size, inside known bounds or even unknown.
#[must_use]
pub fn wire_size(ty: &Type<'_>) -> Option<WireSize> {
    Some(match ty {
        Type::Bool => WireSize::fixed("bool", 1),
        Type::U8 => WireSize::fixed("u8", 1),
        Type::I8 => WireSize::fixed("i8", 1),
        Type::U16 => WireSize::range("u16", 1, 3),
        Type::I16 => WireSize::range("i16", 1, 3),
        Type::U32 => WireSize::range("u32", 1, 5),
        Type::I32 => WireSize::range("i32", 1, 5),
        Type::U64 => WireSize::range("u64", 1, 10),
        Type::I64 => WireSize::range("i64", 1, 10),
        Type::U128 => WireSize::range("u128", 1, 19),
        Type::I128 => WireSize::range("i128", 1, 19),
        Type::F32 => WireSize::fixed("f32", 4),
        Type::F64 => WireSize::fixed("f64", 8),
        Type::String => WireSize::min("string", 1),
        Type::StringRef => WireSize::min("&string", 1),
        Type::Bytes => WireSize::min("bytes", 1),
        Type::BytesRef => WireSize::min("&bytes", 1),
        Type::Vec(ty) => WireSize {
            label: "vec".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&Type::U64)),
                ("element".into(), wire_size(ty)),
            ],
        },
        Type::HashMap(kv) => WireSize {
            label: "hash_map".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&Type::U64)),
                ("key".into(), wire_size(&kv.0)),
                ("value".into(), wire_size(&kv.1)),
            ],
        },
        Type::HashSet(ty) => WireSize {
            label: "hash_set".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&Type::U64)),
                ("element".into(), wire_size(ty)),
            ],
        },
        Type::Option(ty) => {
            let inner = wire_size(ty);
            WireSize {
                label: "option".into(),
                min: 0,
                max: inner.as_ref().and_then(|size| size.max).map(|max| 1 + max),
                inner: vec![("value".into(), inner)],
            }
        }
        Type::NonZero(ty) => {
            let inner = wire_size(ty);
            WireSize {
                label: "non_zero".into(),
                min: 0,
                max: inner.as_ref().and_then(|size| size.max).map(|max| 1 + max),
                inner: vec![("value".into(), inner)],
            }
        }
        Type::BoxString => WireSize::min("box<string>", 1),
        Type::BoxBytes => WireSize::min("box<bytes>", 1),
        Type::Array(ty, size) => wire_size_array(ty, *size),
        Type::Tuple(types) => wire_size_tuple(types),
        Type::External(_) => return None,
    })
}

fn wire_size_array(ty: &Type<'_>, size: u32) -> WireSize {
    let length = varint_size(size);
    let inner = wire_size(ty);

    WireSize {
        label: "array".into(),
        min: length + inner.as_ref().map_or(0, |size| size.min) * size as usize,
        max: inner
            .as_ref()
            .and_then(|size| size.max)
            .map(|max| length + max * size as usize),
        inner: vec![
            ("length".into(), Some(WireSize::fixed("u64", length))),
            ("element".into(), inner),
        ],
    }
}

fn wire_size_tuple(types: &[Type<'_>]) -> WireSize {
    let inner = types
        .iter()
        .enumerate()
        .map(|(i, ty)| (i.to_string().into(), wire_size(ty)))
        .collect::<Vec<_>>();

    WireSize {
        label: "tuple".into(),
        min: inner
            .iter()
            .filter_map(|(_, size)| size.as_ref())
            .map(|size| size.min)
            .sum(),
        max: inner
            .iter()
            .map(|(_, size)| size.as_ref().and_then(|s| s.max))
            .sum(),
        inner,
    }
}

fn varint_size(value: u32) -> usize {
    ((std::mem::size_of::<u32>() * 8 - value.leading_zeros() as usize).div_ceil(7)).max(1)
}
