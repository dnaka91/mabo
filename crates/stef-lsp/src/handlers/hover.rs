use std::{borrow::Cow, fmt::Write, ops::Range};

use anyhow::{Context, Result};
use line_index::{LineIndex, TextSize, WideLineCol};
use lsp_types::{Position, Range as LspRange};
use stef_parser::{
    Comment, Const, DataType, Definition, Enum, Fields, Module, NamedField, Schema, Span, Spanned,
    Struct, Type, TypeAlias, Variant,
};

pub fn visit_schema(
    index: &LineIndex,
    item: &Schema<'_>,
    position: Position,
) -> Result<Option<(String, LspRange)>> {
    let position = index
        .offset(
            index
                .to_utf8(
                    line_index::WideEncoding::Utf16,
                    WideLineCol {
                        line: position.line,
                        col: position.character,
                    },
                )
                .context("missing utf-16 position")?,
        )
        .context("missing offset position")?
        .into();

    item.definitions
        .iter()
        .find_map(|def| visit_definition(def, position))
        .map(|(text, span)| Ok((text, get_range(index, span)?)))
        .transpose()
}

fn visit_definition(item: &Definition<'_>, position: usize) -> Option<(String, Span)> {
    match item {
        Definition::Module(m) => visit_module(m, position),
        Definition::Struct(s) => visit_struct(s, position),
        Definition::Enum(e) => visit_enum(e, position),
        Definition::TypeAlias(a) => visit_alias(a, position),
        Definition::Const(c) => visit_const(c, position),
        Definition::Import(_) => None,
    }
}

fn visit_module(item: &Module<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.name.span()))
        .or_else(|| {
            item.definitions
                .iter()
                .find_map(|def| visit_definition(def, position))
        })
}

fn visit_struct(item: &Struct<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            if let Some(next_id) = next_field_id(&item.fields) {
                let _ = writeln!(&mut text, "- next ID: `{next_id}`");
            }

            (text, item.name.span())
        })
        .or_else(|| visit_fields(&item.fields, position))
}

fn visit_enum(item: &Enum<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            let _ = writeln!(
                &mut text,
                "- next ID: `{}`",
                next_variant_id(&item.variants)
            );

            (text, item.name.span())
        })
        .or_else(|| {
            item.variants
                .iter()
                .find_map(|variant| visit_variant(variant, position))
        })
}

fn visit_variant(item: &Variant<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            if let Some(next_id) = next_field_id(&item.fields) {
                let _ = writeln!(&mut text, "- next ID: `{next_id}`");
            }

            (text, item.name.span())
        })
        .or_else(|| visit_fields(&item.fields, position))
}

fn visit_fields(item: &Fields<'_>, position: usize) -> Option<(String, Span)> {
    if let Fields::Named(named) = item {
        named
            .iter()
            .find_map(|field| visit_named_field(field, position))
    } else {
        None
    }
}

fn visit_named_field(item: &NamedField<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position)).then(|| {
        let mut text = fold_comment(&item.comment);

        let _ = write!(&mut text, "### Wire size\n\n");
        if let Some(size) = wire_size(&item.ty.value) {
            size.print(&mut text, 0);
        } else {
            let _ = write!(&mut text, "_unknown_");
        }

        (text, item.name.span())
    })
}

fn visit_alias(item: &TypeAlias<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.name.span()))
}

fn visit_const(item: &Const<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.name.span()))
}

fn fold_comment(comment: &Comment<'_>) -> String {
    comment.0.iter().fold(String::new(), |mut acc, line| {
        acc.push_str(line.value);
        acc.push('\n');
        acc
    })
}

fn next_variant_id(variants: &[Variant<'_>]) -> u32 {
    variants
        .iter()
        .map(|variant| variant.id.get())
        .max()
        .unwrap_or(0)
        + 1
}

fn next_field_id(fields: &Fields<'_>) -> Option<u32> {
    match fields {
        Fields::Named(named) => {
            Some(named.iter().map(|field| field.id.get()).max().unwrap_or(0) + 1)
        }
        Fields::Unnamed(unnamed) => Some(
            unnamed
                .iter()
                .map(|field| field.id.get())
                .max()
                .unwrap_or(0)
                + 1,
        ),
        Fields::Unit => None,
    }
}

struct WireSize {
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

    fn print(&self, buf: &mut String, indent: usize) {
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

fn wire_size(ty: &DataType<'_>) -> Option<WireSize> {
    Some(match ty {
        DataType::Bool => WireSize::fixed("bool", 1),
        DataType::U8 => WireSize::fixed("u8", 1),
        DataType::I8 => WireSize::fixed("i8", 1),
        DataType::U16 => WireSize::range("u16", 1, 3),
        DataType::I16 => WireSize::range("i16", 1, 3),
        DataType::U32 => WireSize::range("u32", 1, 5),
        DataType::I32 => WireSize::range("i32", 1, 5),
        DataType::U64 => WireSize::range("u64", 1, 10),
        DataType::I64 => WireSize::range("i64", 1, 10),
        DataType::U128 => WireSize::range("u128", 1, 19),
        DataType::I128 => WireSize::range("i128", 1, 19),
        DataType::F32 => WireSize::fixed("f32", 4),
        DataType::F64 => WireSize::fixed("f64", 8),
        DataType::String => WireSize::min("string", 1),
        DataType::StringRef => WireSize::min("&string", 1),
        DataType::Bytes => WireSize::min("bytes", 1),
        DataType::BytesRef => WireSize::min("&bytes", 1),
        DataType::Vec(ty) => WireSize {
            label: "vec".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&DataType::U64)),
                ("element".into(), wire_size(&ty.value)),
            ],
        },
        DataType::HashMap(kv) => WireSize {
            label: "hash_map".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&DataType::U64)),
                ("key".into(), wire_size(&kv.0.value)),
                ("value".into(), wire_size(&kv.1.value)),
            ],
        },
        DataType::HashSet(ty) => WireSize {
            label: "hash_set".into(),
            min: 1,
            max: None,
            inner: vec![
                ("length".into(), wire_size(&DataType::U64)),
                ("element".into(), wire_size(&ty.value)),
            ],
        },
        DataType::Option(ty) => {
            let inner = wire_size(&ty.value);
            WireSize {
                label: "option".into(),
                min: 0,
                max: inner.as_ref().and_then(|size| size.max).map(|max| 1 + max),
                inner: vec![("value".into(), inner)],
            }
        }
        DataType::NonZero(ty) => {
            let inner = wire_size(&ty.value);
            WireSize {
                label: "non_zero".into(),
                min: 0,
                max: inner.as_ref().and_then(|size| size.max).map(|max| 1 + max),
                inner: vec![("value".into(), inner)],
            }
        }
        DataType::BoxString => WireSize::min("box<string>", 1),
        DataType::BoxBytes => WireSize::min("box<bytes>", 1),
        DataType::Array(ty, size) => wire_size_array(ty, *size),
        DataType::Tuple(types) => wire_size_tuple(types),
        DataType::External(_) => return None,
    })
}

fn wire_size_array(ty: &Type<'_>, size: u32) -> WireSize {
    let length = varint_size(size);
    let inner = wire_size(&ty.value);

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
        .map(|(i, ty)| (i.to_string().into(), wire_size(&ty.value)))
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
    ((std::mem::size_of::<u32>() * 8 - value.leading_zeros() as usize + 6) / 7).max(1)
}

#[allow(clippy::cast_possible_truncation)]
fn get_range(index: &LineIndex, span: Span) -> Result<LspRange> {
    let range = Range::from(span);
    let (start, end) = index
        .to_wide(
            line_index::WideEncoding::Utf16,
            index.line_col(TextSize::new(range.start as u32)),
        )
        .zip(index.to_wide(
            line_index::WideEncoding::Utf16,
            index.line_col(TextSize::new(range.end as u32)),
        ))
        .context("missing utf-16 positions")?;

    Ok(LspRange::new(
        Position::new(start.line, start.col),
        Position::new(end.line, end.col),
    ))
}
