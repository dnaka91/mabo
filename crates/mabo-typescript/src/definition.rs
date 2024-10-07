use std::fmt::{self, Display, Write};

use mabo_compiler::simplify::{
    Const, Definition, Enum, ExternalType, Fields, Literal, Schema, Struct, Type, TypeAlias,
    Variant,
};

use crate::{Indent, Opts, Output, decode, encode, size};

/// Take a single schema and convert it into Kotlin source code (which can result in multiple
/// files).
#[must_use]
pub fn render_schema<'a>(
    opts: &'a Opts<'_>,
    Schema { definitions, .. }: &'a Schema<'_>,
) -> Output<'a> {
    let mut content = format!("{RenderImports}");

    let modules = definitions
        .iter()
        .filter_map(|def| render_definition(&mut content, def))
        .collect();

    Output {
        name: opts.package,
        content,
        modules,
    }
}

fn render_definition<'a>(buf: &mut String, definition: &'a Definition<'_>) -> Option<Output<'a>> {
    match definition {
        Definition::Module(m) => {
            let mut content = format!("{RenderImports}");

            let modules = m
                .definitions
                .iter()
                .filter_map(|def| render_definition(&mut content, def))
                .collect();

            return Some(Output {
                name: m.name,
                content,
                modules,
            });
        }
        Definition::Struct(s) => {
            writeln!(buf, "{}", RenderStruct(s)).unwrap();
        }
        Definition::Enum(e) => writeln!(buf, "{}", RenderEnum(e)).unwrap(),
        Definition::TypeAlias(a) => writeln!(buf, "{}", RenderAlias(a)).unwrap(),
        Definition::Const(c) => writeln!(buf, "{}", RenderConst(c)).unwrap(),
        Definition::Import(_) => {}
    }

    None
}

struct RenderImports;

impl Display for RenderImports {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "import * from \"mabo\";\n")
    }
}

struct RenderStruct<'a>(&'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}export class {}{} implements decode.Decode, encode.Encode, size.Size {{",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment
            },
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenerics {
                generics: &self.0.generics,
                fields_filter: None
            },
        )?;

        writeln!(
            f,
            "{}\n{}\n{}\n{}\n{}\n",
            RenderFields {
                indent: Indent(1),
                fields: &self.0.fields,
            },
            RenderConstructor {
                indent: Indent(1),
                fields: &self.0.fields,
            },
            decode::RenderStruct {
                indent: Indent(2),
                item: self.0,
            },
            encode::RenderStruct {
                indent: Indent(1),
                item: self.0,
            },
            size::RenderStruct {
                indent: Indent(1),
                item: self.0,
            },
        )?;

        writeln!(f, "}}")
    }
}

struct RenderEnum<'a>(&'a Enum<'a>);

impl Display for RenderEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "public abstract class {} implements decode.Decode, encode.Encode, size.Size {{",
            heck::AsUpperCamelCase(&self.0.name),
        )?;

        writeln!(
            f,
            "{}\n{}\n",
            encode::RenderEnum {
                indent: Indent(1),
                item: self.0
            },
            size::RenderEnum {
                indent: Indent(1),
                item: self.0
            }
        )?;

        for variant in &self.0.variants {
            write!(
                f,
                "\n{}",
                RenderEnumVariant {
                    enum_name: self.0.name,
                    generics: &self.0.generics,
                    variant
                }
            )?;
        }

        writeln!(f, "}}")
    }
}

struct RenderEnumVariant<'a> {
    enum_name: &'a str,
    generics: &'a [&'a str],
    variant: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}{}export class {}{} extends {} implements decode.Decode, encode.Encode, size.Size \
             {{",
            RenderComment {
                indent: Indent(1),
                comment: &self.variant.comment
            },
            Indent(1),
            heck::AsUpperCamelCase(&self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenerics {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields)
            },
        )?;

        writeln!(
            f,
            "{}\n{}\n{}\n",
            decode::RenderEnumVariant {
                indent: Indent(3),
                generics: self.generics,
                item: self.variant,
            },
            encode::RenderEnumVariant {
                indent: Indent(2),
                item: self.variant,
            },
            size::RenderEnumVariant {
                indent: Indent(2),
                item: self.variant,
            },
        )?;

        writeln!(f, "{}}}", Indent(1))
    }
}

struct RenderFields<'a> {
    indent: Indent,
    fields: &'a Fields<'a>,
}

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, fields } = *self;
        if fields.fields.is_empty() {
            return Ok(());
        }

        for field in &*fields.fields {
            writeln!(
                f,
                "{}{indent}{}: {}",
                RenderComment {
                    indent,
                    comment: &field.comment
                },
                heck::AsLowerCamelCase(&field.name),
                RenderType(&field.ty)
            )?;
        }
        Ok(())
    }
}

struct RenderConstructor<'a> {
    indent: Indent,
    fields: &'a Fields<'a>,
}

impl Display for RenderConstructor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, fields } = *self;

        writeln!(f, "{indent}constructor(")?;
        for field in &*fields.fields {
            writeln!(
                f,
                "{indent}{}: {} = 0,",
                heck::AsLowerCamelCase(&field.name),
                RenderType(&field.ty),
                indent = indent + 1,
            )?;
        }
        writeln!(f, "{indent}) {{")?;
        for field in &*fields.fields {
            writeln!(
                f,
                "{indent}this.{} = {0};",
                heck::AsLowerCamelCase(&field.name),
                indent = indent + 1,
            )?;
        }
        writeln!(f, "{indent}}}")
    }
}

pub(super) struct RenderGenerics<'a> {
    pub generics: &'a [&'a str],
    pub fields_filter: Option<&'a Fields<'a>>,
}

impl Display for RenderGenerics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.fields_filter {
            Some(fields) => {
                if !self.generics.iter().any(|g| uses_generic(g, fields)) {
                    return Ok(());
                }
            }
            None => {
                if self.generics.is_empty() {
                    return Ok(());
                }
            }
        }

        f.write_char('<')?;
        for (i, value) in self
            .generics
            .iter()
            .filter(|g| match self.fields_filter {
                Some(fields) => uses_generic(g, fields),
                None => true,
            })
            .enumerate()
        {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{value}")?;
        }
        f.write_char('>')
    }
}

struct RenderAlias<'a>(&'a TypeAlias<'a>);

impl Display for RenderAlias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}type {}{} = {};",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment,
            },
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenerics {
                generics: &self.0.generics,
                fields_filter: None
            },
            RenderType(&self.0.target),
        )
    }
}

struct RenderConst<'a>(&'a Const<'a>);

impl Display for RenderConst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}const {}: {} = {};",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment
            },
            heck::AsShoutySnakeCase(&self.0.name),
            RenderConstType(&self.0.ty),
            RenderLiteral(&self.0.value),
        )
    }
}

struct RenderConstType<'a>(&'a Type<'a>);

impl Display for RenderConstType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "boolean"),
            Type::U8
            | Type::U16
            | Type::U32
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::F32
            | Type::F64 => write!(f, "number"),
            Type::U64 | Type::U128 | Type::I64 | Type::I128 => write!(f, "bigint"),
            Type::String | Type::StringRef => write!(f, "string"),
            Type::Bytes | Type::BytesRef => write!(f, "Uint8Array"),
            _ => panic!("invalid data type for const"),
        }
    }
}

struct RenderLiteral<'a>(&'a Literal);

impl Display for RenderLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Int(i) => write!(f, "{i}"),
            Literal::Float(f2) => write!(f, "{f2}"),
            Literal::String(s) => write!(f, "{s:?}"),
            Literal::Bytes(b) => {
                f.write_str("new Uint8Array([")?;
                for (i, value) in b.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    value.fmt(f)?;
                }
                f.write_str("])")
            }
        }
    }
}

struct RenderComment<'a> {
    indent: Indent,
    comment: &'a [&'a str],
}

impl Display for RenderComment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, comment } = *self;
        if self.comment.is_empty() {
            return Ok(());
        }

        writeln!(f, "{indent}/**")?;
        for line in comment {
            writeln!(f, "{indent} * {line}")?;
        }
        writeln!(f, "{indent} */")
    }
}

pub(super) struct RenderType<'a>(pub(super) &'a Type<'a>);

impl Display for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "boolean"),
            Type::U8
            | Type::U16
            | Type::U32
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::F32
            | Type::F64 => write!(f, "number"),
            Type::U64 | Type::U128 | Type::I64 | Type::I128 => write!(f, "bigint"),
            Type::String | Type::StringRef | Type::BoxString => write!(f, "string"),
            Type::Bytes | Type::BytesRef | Type::BoxBytes => write!(f, "Uint8Array"),
            Type::Vec(ty) => write!(f, "List<{}>", RenderType(ty)),
            Type::HashMap(kv) => write!(f, "Map<{}, {}>", RenderType(&kv.0), RenderType(&kv.1)),
            Type::HashSet(ty) => write!(f, "Set<{}>", RenderType(ty)),
            Type::Option(ty) => write!(f, "{} | undefined", RenderType(ty)),
            Type::NonZero(ty) => match &**ty {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::F32
                | Type::F64 => write!(f, "NonZeroNumber"),
                Type::U64 | Type::U128 | Type::I64 | Type::I128 => write!(f, "NonZeroBigInt"),
                Type::String | Type::StringRef => write!(f, "NonZeroString"),
                Type::Bytes | Type::BytesRef => write!(f, "NonZeroBytes"),
                Type::Vec(ty) => write!(f, "NonZeroVec<{}>", RenderType(ty)),
                Type::HashMap(kv) => write!(
                    f,
                    "NonZeroHashMap<{}, {}>",
                    RenderType(&kv.0),
                    RenderType(&kv.1)
                ),
                Type::HashSet(ty) => write!(f, "NonZeroHashSet<{}>", RenderType(ty)),
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => write!(f, "[{}]", Concat(types)),
            Type::Array(ty, _) => write!(f, "Array<{}>", RenderType(ty)),
            Type::External(ExternalType { name, generics, .. }) => {
                write!(
                    f,
                    "{}{}",
                    heck::AsUpperCamelCase(name),
                    RenderGenericTypes(generics),
                )
            }
        }
    }
}

struct Concat<'a>(&'a [Type<'a>]);

impl Display for Concat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            RenderType(value).fmt(f)?;
        }

        Ok(())
    }
}

struct RenderGenericTypes<'a>(&'a [Type<'a>]);

impl Display for RenderGenericTypes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        f.write_char('<')?;
        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", RenderType(value))?;
        }
        f.write_char('>')
    }
}

fn uses_generic(generic: &str, fields: &Fields<'_>) -> bool {
    fn visit_external(ty: &Type<'_>, visit: &impl Fn(&ExternalType<'_>) -> bool) -> bool {
        match ty {
            Type::Bool
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::F32
            | Type::F64
            | Type::String
            | Type::StringRef
            | Type::Bytes
            | Type::BytesRef
            | Type::BoxString
            | Type::BoxBytes => false,
            Type::Vec(ty)
            | Type::HashSet(ty)
            | Type::Option(ty)
            | Type::NonZero(ty)
            | Type::Array(ty, _) => visit_external(ty, visit),
            Type::HashMap(kv) => visit_external(&kv.0, visit) || visit_external(&kv.1, visit),
            Type::Tuple(types) => types.iter().any(|ty| visit_external(ty, visit)),
            Type::External(ty) => visit(ty),
        }
    }

    let matches = |ext: &ExternalType<'_>| {
        ext.path.is_empty() && ext.generics.is_empty() && ext.name == generic
    };

    fields
        .fields
        .iter()
        .any(|field| visit_external(&field.ty, &matches))
}
