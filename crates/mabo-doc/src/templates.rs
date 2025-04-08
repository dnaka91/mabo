use std::{
    fmt::{self, Display, Write},
    rc::Rc,
};

use askama::Template;
use mabo_compiler::simplify::{
    Const, Definition, Enum, ExternalType, Field, FieldKind, Literal, Module, Struct, Type,
    TypeAlias,
};
use mabo_meta::WireSize;

mod filters {
    #![expect(clippy::unnecessary_wraps)]

    use askama::filters::Safe;
    use comrak::{ExtensionOptions, ParseOptions, RenderOptions};

    pub fn markdown(s: impl AsRef<str>, _: &dyn askama::Values) -> askama::Result<Safe<String>> {
        let extension = ExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            footnotes: true,
            ..ExtensionOptions::default()
        };

        let parse = ParseOptions::default();

        let render = RenderOptions {
            escape: true,
            ..RenderOptions::default()
        };

        Ok(Safe(comrak::markdown_to_html(
            s.as_ref(),
            &comrak::Options {
                extension,
                parse,
                render,
            },
        )))
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    pub name: &'a str,
    pub path: &'a [Rc<str>],
    pub comment: &'a [&'a str],
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

impl StructDetail<'_> {
    fn print_schema(&self) -> String {
        let mut buf = format!("struct {}", self.item.name);

        if !self.item.generics.is_empty() {
            buf.push('<');
            for (i, g) in self.item.generics.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(g);
            }
            buf.push('>');
        }

        match self.item.fields.kind {
            FieldKind::Named => {
                buf.push_str(" {\n");
                for field in &*self.item.fields.fields {
                    let _ = writeln!(&mut buf, "    {},", PrintField(field, FieldKind::Named));
                }
                buf.push('}');
            }
            FieldKind::Unnamed => {
                buf.push('(');
                for (i, field) in self.item.fields.fields.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    let _ = write!(&mut buf, "{}", PrintField(field, FieldKind::Unnamed));
                }
                buf.push(')');
            }
            FieldKind::Unit => {}
        }

        buf
    }
}

#[derive(Template)]
#[template(path = "detail/enum.html")]
pub struct EnumDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Enum<'a>,
}

impl EnumDetail<'_> {
    fn print_schema(&self) -> String {
        let mut buf = format!("enum {}", self.item.name);

        if !self.item.generics.is_empty() {
            buf.push('<');
            for (i, g) in self.item.generics.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(g);
            }
            buf.push('>');
        }

        buf.push_str(" {\n");

        for variant in &self.item.variants {
            let _ = write!(&mut buf, "    {}", variant.name);
            match variant.fields.kind {
                FieldKind::Named => {
                    buf.push_str(" {\n");
                    for field in &*variant.fields.fields {
                        let _ =
                            writeln!(&mut buf, "        {},", PrintField(field, FieldKind::Named));
                    }
                    let _ = writeln!(&mut buf, "    }} @{},", variant.id);
                }
                FieldKind::Unnamed => {
                    buf.push('(');
                    for (i, field) in variant.fields.fields.iter().enumerate() {
                        if i > 0 {
                            buf.push_str(", ");
                        }
                        let _ = write!(&mut buf, "{}", PrintField(field, FieldKind::Unnamed));
                    }
                    let _ = writeln!(&mut buf, ") @{},", variant.id);
                }
                FieldKind::Unit => {
                    let _ = writeln!(&mut buf, " @{},", variant.id);
                }
            }
        }

        buf.push('}');
        buf
    }
}

#[derive(Template)]
#[template(path = "detail/alias.html")]
pub struct AliasDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a TypeAlias<'a>,
}

impl AliasDetail<'_> {
    fn print_schema(&self) -> String {
        let mut buf = format!("type {}", self.item.name);

        if !self.item.generics.is_empty() {
            buf.push('<');
            for (i, g) in self.item.generics.iter().enumerate() {
                if i > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(g);
            }
            buf.push('>');
        }

        let _ = write!(&mut buf, " = {};", PrintType(&self.item.target));
        buf
    }
}

#[derive(Template)]
#[template(path = "detail/const.html")]
pub struct ConstDetail<'a> {
    pub path: &'a [Rc<str>],
    pub item: &'a Const<'a>,
}

impl ConstDetail<'_> {
    fn print_schema(&self) -> String {
        format!(
            "const {}: {} = {};",
            self.item.name,
            PrintType(&self.item.ty),
            PrintLiteral(&self.item.value)
        )
    }
}

fn render_wire_size(size: &WireSize) -> String {
    let mut buf = String::new();
    size.print(&mut buf, 0);
    buf
}

#[expect(clippy::trivially_copy_pass_by_ref)]
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

fn first_comment(item: &[&str]) -> String {
    item.iter()
        .take_while(|line| !line.trim().is_empty())
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        })
}

fn merge_comments(item: &[&str]) -> String {
    item.iter().fold(String::new(), |mut acc, line| {
        acc.push_str(line);
        acc.push('\n');
        acc
    })
}

struct PrintField<'a>(&'a Field<'a>, FieldKind);

impl Display for PrintField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            FieldKind::Named => write!(
                f,
                "{}: {} @{}",
                self.0.name,
                PrintType(&self.0.ty),
                self.0.id
            ),
            FieldKind::Unnamed => write!(f, "{} @{}", PrintType(&self.0.ty), self.0.id),
            FieldKind::Unit => Ok(()),
        }
    }
}

struct PrintType<'a>(&'a Type<'a>);

impl Display for PrintType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => f.write_str("bool"),
            Type::U8 => f.write_str("u8"),
            Type::U16 => f.write_str("u16"),
            Type::U32 => f.write_str("u32"),
            Type::U64 => f.write_str("u64"),
            Type::U128 => f.write_str("u128"),
            Type::I8 => f.write_str("i8"),
            Type::I16 => f.write_str("i16"),
            Type::I32 => f.write_str("i32"),
            Type::I64 => f.write_str("i64"),
            Type::I128 => f.write_str("i128"),
            Type::F32 => f.write_str("f32"),
            Type::F64 => f.write_str("f64"),
            Type::String => f.write_str("string"),
            Type::StringRef => f.write_str("&string"),
            Type::Bytes => f.write_str("bytes"),
            Type::BytesRef => f.write_str("&bytes"),
            Type::Vec(t) => write!(f, "vec<{}>", Self(t)),
            Type::HashMap(kv) => write!(f, "hash_map<{}, {}>", Self(&kv.0), Self(&kv.1)),
            Type::HashSet(t) => write!(f, "hash_set<{}>", Self(t)),
            Type::Option(t) => write!(f, "option<{}>", Self(t)),
            Type::NonZero(t) => write!(f, "non_zero<{}>", Self(t)),
            Type::BoxString => f.write_str("box<string>"),
            Type::BoxBytes => f.write_str("box<bytes>"),
            Type::Tuple(types) => {
                f.write_char('(')?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}", PrintType(ty))?;
                }
                f.write_char(')')
            }
            Type::Array(t, size) => write!(f, "[{}; {size}]", Self(t)),
            Type::External(ExternalType {
                path,
                name,
                generics,
            }) => {
                for seg in &**path {
                    write!(f, "{seg}::")?;
                }
                f.write_str(name)?;

                if !generics.is_empty() {
                    f.write_char('<')?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            f.write_str(", ")?;
                        }
                        write!(f, "{}", PrintType(g))?;
                    }
                    f.write_char('>')?;
                }

                Ok(())
            }
        }
    }
}

struct PrintLiteral<'a>(&'a Literal);

impl Display for PrintLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Literal::Bool(v) => v.fmt(f),
            Literal::Int(v) => v.fmt(f),
            Literal::Float(v) => v.fmt(f),
            Literal::String(v) => write!(f, "{v:?}"),
            Literal::Bytes(v) => write!(f, "{v:?}"),
        }
    }
}
