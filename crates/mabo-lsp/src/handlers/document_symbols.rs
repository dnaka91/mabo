use std::{fmt::Write, ops::Range};

use anyhow::Result;
use lsp_types::{self as lsp, DocumentSymbol, SymbolKind};
use mabo_parser::{
    Const, Definition, Enum, Fields, Import, Module, NamedField, Schema, Spanned, Struct,
    TypeAlias, UnnamedField, Variant,
};

use super::index::Index;

pub fn visit_schema(index: &Index, item: &Schema<'_>) -> Result<Vec<DocumentSymbol>> {
    item.definitions
        .iter()
        .map(|def| visit_definition(index, def))
        .collect()
}

fn visit_definition(index: &Index, item: &Definition<'_>) -> Result<DocumentSymbol> {
    match item {
        Definition::Module(m) => visit_module(index, m),
        Definition::Struct(s) => visit_struct(index, s),
        Definition::Enum(e) => visit_enum(index, e),
        Definition::TypeAlias(a) => visit_alias(index, a),
        Definition::Const(c) => visit_const(index, c),
        Definition::Import(i) => visit_import(index, i),
    }
}

fn visit_module(index: &Index, item: &Module<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::MODULE,
        index.get_range(item.name.span())?,
        item.definitions
            .iter()
            .map(|def| visit_definition(index, def))
            .collect::<Result<_>>()?,
    ))
}

fn visit_struct(index: &Index, item: &Struct<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::STRUCT,
        index.get_range(item.name.span())?,
        visit_fields(index, &item.fields)?,
    ))
}

fn visit_enum(index: &Index, item: &Enum<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::ENUM,
        index.get_range(item.name.span())?,
        item.variants
            .iter()
            .map(|variant| visit_variant(index, variant))
            .collect::<Result<_>>()?,
    ))
}

fn visit_variant(index: &Index, item: &Variant<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::ENUM_MEMBER,
        index.get_range(item.name.span())?,
        visit_fields(index, &item.fields)?,
    ))
}

fn visit_fields(index: &Index, item: &Fields<'_>) -> Result<Vec<DocumentSymbol>> {
    match item {
        Fields::Named(_, named) => named
            .iter()
            .map(|field| visit_named_field(index, field))
            .collect(),
        Fields::Unnamed(_, unnamed) => unnamed
            .iter()
            .enumerate()
            .map(|(pos, field)| visit_unnamed_field(index, field, pos))
            .collect(),
        Fields::Unit => Ok(vec![]),
    }
}

fn visit_named_field(index: &Index, item: &NamedField<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::PROPERTY,
        index.get_range(item.name.span())?,
        vec![],
    ))
}

fn visit_unnamed_field(
    index: &Index,
    item: &UnnamedField<'_>,
    pos: usize,
) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        &pos.to_string(),
        SymbolKind::PROPERTY,
        index.get_range(item.span())?,
        vec![],
    ))
}

fn visit_alias(index: &Index, item: &TypeAlias<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::VARIABLE,
        index.get_range(item.name.span())?,
        vec![],
    ))
}

fn visit_const(index: &Index, item: &Const<'_>) -> Result<DocumentSymbol> {
    Ok(create_symbol(
        item.name.get(),
        SymbolKind::CONSTANT,
        index.get_range(item.name.span())?,
        vec![],
    ))
}

fn visit_import(index: &Index, item: &Import<'_>) -> Result<DocumentSymbol> {
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
        index.get_range(span)?,
        vec![],
    ))
}

#[allow(deprecated)]
fn create_symbol(
    name: &str,
    kind: SymbolKind,
    range: lsp::Range,
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
