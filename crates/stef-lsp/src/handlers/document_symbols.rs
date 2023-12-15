use std::{fmt::Write, ops::Range};

use anyhow::{Context, Result};
use line_index::{LineIndex, TextSize};
use lsp_types::{DocumentSymbol, Position, Range as LspRange, SymbolKind};
use stef_parser::{
    Const, Definition, Enum, Fields, Import, Module, NamedField, Schema, Span, Spanned, Struct,
    TypeAlias, UnnamedField, Variant,
};

pub fn visit_schema(index: &LineIndex, item: &Schema<'_>) -> Result<Vec<DocumentSymbol>> {
    item.definitions
        .iter()
        .map(|def| visit_definition(index, def))
        .collect()
}

fn visit_definition(index: &LineIndex, item: &Definition<'_>) -> Result<DocumentSymbol> {
    match item {
        Definition::Module(m) => visit_module(index, m),
        Definition::Struct(s) => visit_struct(index, s),
        Definition::Enum(e) => visit_enum(index, e),
        Definition::TypeAlias(a) => visit_alias(index, a),
        Definition::Const(c) => visit_const(index, c),
        Definition::Import(i) => visit_import(index, i),
    }
}

fn visit_module(index: &LineIndex, item: &Module<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::MODULE,
        get_range(index, item.name.span())?,
        item.definitions
            .iter()
            .map(|def| visit_definition(index, def))
            .collect::<Result<_>>()?,
    ))
}

fn visit_struct(index: &LineIndex, item: &Struct<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::STRUCT,
        get_range(index, item.name.span())?,
        visit_fields(index, &item.fields)?,
    ))
}

fn visit_enum(index: &LineIndex, item: &Enum<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::ENUM,
        get_range(index, item.name.span())?,
        item.variants
            .iter()
            .map(|variant| visit_variant(index, variant))
            .collect::<Result<_>>()?,
    ))
}

fn visit_variant(index: &LineIndex, item: &Variant<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::ENUM_MEMBER,
        get_range(index, item.name.span())?,
        visit_fields(index, &item.fields)?,
    ))
}

fn visit_fields(index: &LineIndex, item: &Fields<'_>) -> Result<Vec<DocumentSymbol>> {
    match item {
        Fields::Named(named) => named
            .iter()
            .map(|field| visit_named_field(index, field))
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .iter()
            .enumerate()
            .map(|(pos, field)| visit_unnamed_field(index, field, pos))
            .collect(),
        Fields::Unit => Ok(vec![]),
    }
}

fn visit_named_field(index: &LineIndex, item: &NamedField<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::PROPERTY,
        get_range(index, item.name.span())?,
        vec![],
    ))
}

fn visit_unnamed_field(
    index: &LineIndex,
    item: &UnnamedField<'_>,
    pos: usize,
) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        &pos.to_string(),
        SymbolKind::PROPERTY,
        get_range(index, item.span())?,
        vec![],
    ))
}

fn visit_alias(index: &LineIndex, item: &TypeAlias<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::VARIABLE,
        get_range(index, item.name.span())?,
        vec![],
    ))
}

fn visit_const(index: &LineIndex, item: &Const<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::CONSTANT,
        get_range(index, item.name.span())?,
        vec![],
    ))
}

fn visit_import(index: &LineIndex, item: &Import<'_>) -> Result<DocumentSymbol> {
    debug_assert!(
        !item.segments.is_empty(),
        "there should always be at least one segment"
    );

    let mut name = item.segments[0].get().to_owned();
    let mut span = Range::from(item.segments[0].span());

    for segment in item
        .segments
        .iter()
        .skip(1)
        .chain(std::iter::once(&item.element).flatten())
    {
        let _ = write!(&mut name, "::{segment}");
        span.end = Range::from(segment.span()).end;
    }

    Ok(create_symbol(
        &name,
        SymbolKind::FILE,
        get_range(index, span.into())?,
        vec![],
    ))
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

#[allow(deprecated)]
fn create_symbol(
    name: &str,
    kind: SymbolKind,
    range: LspRange,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol {
    DocumentSymbol {
        name: name.to_owned(),
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    }
}
