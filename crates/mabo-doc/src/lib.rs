//! Documentation generator for Mabo schema files, akin to Rust's `rustdoc`.

use std::rc::Rc;

use anyhow::Result;
use askama::Template;
use mabo_compiler::simplify::{Alias, Const, Definition, Enum, Module, Schema, Struct};

mod templates;

/// Additional options to modify the behavior of the documentation generation.
pub struct Opts {}

/// The in-memory output of rendering a single [`Schema`] into HTML documentation.
#[derive(Debug)]
pub struct Output<'a> {
    /// Name of the element, without any module path attached to it. For example, the plain struct
    /// or module name.
    pub name: &'a str,
    /// Module path from the root of the schema to the current element/file.
    pub path: Rc<[Rc<str>]>,
    /// Name of the output file. This might contain a parent directly name as well, in case it is a
    /// module as each module is grouped into its own subfolder together with all its definitions.
    pub file: String,
    /// Fully generated content of the file that should be written to disk.
    pub content: String,
    /// Sub-modules of the root of the schema or currently represented module within.
    pub modules: Vec<Output<'a>>,
}

/// Convert the given schema into a tree of HTML documentation files.
///
/// The files are all kept in memory and writing to disc is up to the caller.
///
/// # Errors
///
/// Will return `Err` if any of the HTML templates fail to render.
pub fn render_schema<'a>(
    _opts: &'a Opts,
    Schema {
        source,
        comment,
        definitions,
    }: &'a Schema<'_>,
) -> Result<Output<'a>> {
    let name = source
        .path
        .as_ref()
        .and_then(|p| p.file_stem())
        .and_then(|p| p.to_str())
        .unwrap_or("root");
    let path = Rc::from([Rc::from(name)]);

    Ok(Output {
        content: templates::Index {
            name,
            path: &path,
            comment,
            definitions,
        }
        .render()?,
        modules: definitions
            .iter()
            .filter_map(|def| render_definition(def, &path))
            .collect::<Result<_>>()?,
        name,
        path,
        file: format!("{name}/index.html"),
    })
}

fn render_definition<'a>(
    item: &'a Definition<'_>,
    path: &Rc<[Rc<str>]>,
) -> Option<Result<Output<'a>>> {
    Some(match item {
        Definition::Module(m) => render_module(m, path),
        Definition::Struct(s) => render_struct(s, path),
        Definition::Enum(e) => render_enum(e, path),
        Definition::Alias(a) => render_alias(a, path),
        Definition::Const(c) => render_const(c, path),
        Definition::Import(_) => return None,
    })
}

fn render_module<'a>(item: &'a Module<'_>, path: &Rc<[Rc<str>]>) -> Result<Output<'a>> {
    let path = {
        let mut path = path.to_vec();
        path.push(item.name.into());
        Rc::from(path)
    };

    Ok(Output {
        content: templates::ModuleDetail { path: &path, item }.render()?,
        modules: item
            .definitions
            .iter()
            .filter_map(|def| render_definition(def, &path))
            .collect::<Result<_>>()?,
        name: item.name,
        path,
        file: format!("{}/index.html", item.name),
    })
}

fn render_struct<'a>(item: &'a Struct<'_>, path: &Rc<[Rc<str>]>) -> Result<Output<'a>> {
    Ok(Output {
        name: item.name,
        path: Rc::clone(path),
        file: format!("struct.{}.html", item.name),
        content: templates::StructDetail { path, item }.render()?,
        modules: Vec::new(),
    })
}

fn render_enum<'a>(item: &'a Enum<'_>, path: &Rc<[Rc<str>]>) -> Result<Output<'a>> {
    Ok(Output {
        name: item.name,
        path: Rc::clone(path),
        file: format!("enum.{}.html", item.name),
        content: templates::EnumDetail { path, item }.render()?,
        modules: Vec::new(),
    })
}

fn render_alias<'a>(item: &'a Alias<'_>, path: &Rc<[Rc<str>]>) -> Result<Output<'a>> {
    Ok(Output {
        name: item.name,
        path: Rc::clone(path),
        file: format!("alias.{}.html", item.name),
        content: templates::AliasDetail { path, item }.render()?,
        modules: Vec::new(),
    })
}

fn render_const<'a>(item: &'a Const<'_>, path: &Rc<[Rc<str>]>) -> Result<Output<'a>> {
    Ok(Output {
        name: item.name,
        path: Rc::clone(path),
        file: format!("constant.{}.html", item.name),
        content: templates::ConstDetail { path, item }.render()?,
        modules: Vec::new(),
    })
}
