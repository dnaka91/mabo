use std::{
    fmt::{self, Display},
    rc::Rc,
};

use askama::Template;
use stef_meta::WireSize;
use stef_parser::{Comment, Const, Definition, Enum, Fields, Module, Struct, TypeAlias};

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub name: &'a str,
    pub path: &'a [Rc<str>],
    pub definitions: &'a [Definition<'a>],
}

#[derive(Template)]
#[template(path = "detail/module.html")]
pub struct ModuleDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Module<'a>,
}

#[derive(Template)]
#[template(path = "detail/struct.html")]
pub struct StructDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Struct<'a>,
}

#[derive(Template)]
#[template(path = "detail/enum.html")]
pub struct EnumDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Enum<'a>,
}

#[derive(Template)]
#[template(path = "detail/alias.html")]
pub struct AliasDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a TypeAlias<'a>,
}

#[derive(Template)]
#[template(path = "detail/const.html")]
pub struct ConstDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Const<'a>,
}

fn render_wire_size(size: &WireSize) -> String {
    let mut buf = String::new();
    size.print(&mut buf, 0);
    buf
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn path_up(len: usize, i: &usize) -> PathUp {
    debug_assert!(len > 0);
    debug_assert!(len > *i);
    PathUp(len - *i - 1)
}

struct PathUp(usize);

impl Display for PathUp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            f.write_str("../")?;
        }
        Ok(())
    }
}

fn first_comment(item: &Comment<'_>) -> String {
    item.0
        .iter()
        .take_while(|line| !line.value.trim().is_empty())
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line.value);
            acc.push('\n');
            acc
        })
}

fn merge_comments(item: &Comment<'_>) -> String {
    item.0.iter().fold(String::new(), |mut acc, line| {
        acc.push_str(line.value);
        acc.push('\n');
        acc
    })
}

struct MergeComments<'a>(&'a Comment<'a>);

impl Display for MergeComments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.0 .0 {
            writeln!(f, "{}", line.value)?;
        }
        Ok(())
    }
}
