use std::{fmt::Write, ops::Range};

use anyhow::Result;
use lsp_types as lsp;
use mabo_compiler::simplify::{
    Const, Definition, Enum, Field, Fields, Module, ParserField, Schema, Struct, TypeAlias, Variant,
};
use mabo_parser::{Span, Spanned};

use super::index::Index;
use crate::config;

pub fn visit_schema(
    config: &config::Hover,
    index: &Index,
    item: &Schema<'_>,
    position: lsp::Position,
) -> Result<Option<(String, lsp::Range)>> {
    let position = index.get_offset(position)?;

    item.definitions
        .iter()
        .find_map(|def| visit_definition(config, def, position))
        .map(|(text, span)| Ok((text, index.get_range(span)?)))
        .transpose()
}

fn visit_definition(
    config: &config::Hover,
    item: &Definition<'_>,
    position: usize,
) -> Option<(String, Span)> {
    match item {
        Definition::Module(m) => visit_module(config, m, position),
        Definition::Struct(s) => visit_struct(config, s, position),
        Definition::Enum(e) => visit_enum(config, e, position),
        Definition::TypeAlias(a) => visit_alias(a, position),
        Definition::Const(c) => visit_const(c, position),
        Definition::Import(_) => None,
    }
}

fn visit_module(
    config: &config::Hover,
    item: &Module<'_>,
    position: usize,
) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.source.name.span()))
        .or_else(|| {
            item.definitions
                .iter()
                .find_map(|def| visit_definition(config, def, position))
        })
}

fn visit_struct(
    config: &config::Hover,
    item: &Struct<'_>,
    position: usize,
) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            if let Some(next_id) = config
                .show_next_id
                .then(|| mabo_meta::next_field_id(&item.fields))
                .flatten()
            {
                let _ = writeln!(&mut text, "- next ID: `{next_id}`");
            }

            (text, item.source.name.span())
        })
        .or_else(|| visit_fields(config, &item.fields, position))
}

fn visit_enum(config: &config::Hover, item: &Enum<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            if config.show_next_id {
                let _ = writeln!(
                    &mut text,
                    "- next ID: `{}`",
                    mabo_meta::next_variant_id(&item.variants)
                );
            }

            (text, item.source.name.span())
        })
        .or_else(|| {
            item.variants
                .iter()
                .find_map(|variant| visit_variant(config, variant, position))
        })
}

fn visit_variant(
    config: &config::Hover,
    item: &Variant<'_>,
    position: usize,
) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| {
            let mut text = fold_comment(&item.comment);

            if let Some(next_id) = config
                .show_next_id
                .then(|| mabo_meta::next_field_id(&item.fields))
                .flatten()
            {
                let _ = writeln!(&mut text, "- next ID: `{next_id}`");
            }

            (text, item.source.name.span())
        })
        .or_else(|| visit_fields(config, &item.fields, position))
}

fn visit_fields(
    config: &config::Hover,
    item: &Fields<'_>,
    position: usize,
) -> Option<(String, Span)> {
    item.fields
        .iter()
        .find_map(|field| visit_named_field(config, field, position))
}

fn visit_named_field(
    config: &config::Hover,
    item: &Field<'_>,
    position: usize,
) -> Option<(String, Span)> {
    let ParserField::Named(field) = item.source else {
        return None;
    };

    (Range::from(field.name.span()).contains(&position)).then(|| {
        let mut text = fold_comment(&item.comment);

        if config.show_wire_size {
            let _ = write!(&mut text, "### Wire size\n\n");
            if let Some(size) = mabo_meta::wire_size(&item.ty) {
                size.print(&mut text, 0);
            } else {
                let _ = write!(&mut text, "_unknown_");
            }
        }

        (text, field.name.span())
    })
}

fn visit_alias(item: &TypeAlias<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.source.name.span()))
}

fn visit_const(item: &Const<'_>, position: usize) -> Option<(String, Span)> {
    (Range::from(item.source.name.span()).contains(&position))
        .then(|| (fold_comment(&item.comment), item.source.name.span()))
}

fn fold_comment(comment: &[&str]) -> String {
    comment.iter().fold(String::new(), |mut acc, line| {
        acc.push_str(line);
        acc.push('\n');
        acc
    })
}
