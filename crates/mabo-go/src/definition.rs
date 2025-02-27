use std::fmt::{self, Display, Write};

use mabo_compiler::simplify::{
    Const, Definition, Enum, ExternalType, Fields, Literal, Schema, Struct, Type, TypeAlias,
    Variant,
};

use crate::{Indent, Opts, Output, decode, encode, size};

/// Take a single schema and convert it into Go source code (which can result in multiple files).
#[must_use]
pub fn render_schema<'a>(
    opts: &'a Opts<'_>,
    Schema { definitions, .. }: &'a Schema<'_>,
) -> Output<'a> {
    let mut content = format!(
        "{}{}{}",
        RenderHeader,
        RenderPackage(opts.package, None),
        RenderImports,
    );

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
            let mut content = format!(
                "{}{}{}",
                RenderHeader,
                RenderPackage(m.name, Some(&m.comment)),
                RenderImports,
            );

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
            writeln!(
                buf,
                "{}",
                RenderNewFunc {
                    name: heck::AsUpperCamelCase(&s.name),
                    generics: &s.generics,
                    fields: &s.fields,
                    filter_generics: false,
                }
            )
            .unwrap();
            writeln!(
                buf,
                "\n{}\n{}\n{}",
                encode::RenderStruct(s),
                decode::RenderStruct(s),
                size::RenderStruct(s),
            )
            .unwrap();
        }
        Definition::Enum(e) => writeln!(buf, "{}", RenderEnum(e)).unwrap(),
        Definition::TypeAlias(a) => writeln!(buf, "{}", RenderAlias(a)).unwrap(),
        Definition::Const(c) => write!(buf, "{}", RenderConst(c)).unwrap(),
        Definition::Import(_) => {}
    }

    None
}

struct RenderHeader;

impl Display for RenderHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "// Code generated by {} (v{}). DO NOT EDIT.\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        )
    }
}

struct RenderPackage<'a>(&'a str, Option<&'a [&'a str]>);

impl Display for RenderPackage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = self.1 {
            write!(
                f,
                "{}",
                RenderComment {
                    indent: Indent(0),
                    comment
                }
            )?;
        }

        writeln!(f, "package {}\n", heck::AsSnakeCase(self.0))
    }
}

struct RenderImports;

impl Display for RenderImports {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "import (")?;
        writeln!(f, "\tmabo \"github.com/dnaka91/mabo-go\"")?;
        writeln!(f, "\tbuf \"github.com/dnaka91/mabo-go/buf\"")?;
        writeln!(f, ")\n")
    }
}

struct RenderStruct<'a>(&'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}type {}{} {}",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment
            },
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenerics {
                generics: &self.0.generics,
                fields_filter: None
            },
            RenderFields(&self.0.fields)
        )
    }
}

struct RenderNewFunc<'a, T> {
    name: T,
    generics: &'a [&'a str],
    fields: &'a Fields<'a>,
    filter_generics: bool,
}

impl<T> Display for RenderNewFunc<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "func New{}{}({}) {0}{} {{",
            self.name,
            RenderGenerics {
                generics: self.generics,
                fields_filter: self.filter_generics.then_some(self.fields)
            },
            RenderParameters(self.fields),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: self.filter_generics.then_some(self.fields)
            },
        )?;
        writeln!(
            f,
            "\treturn {}{}{}",
            self.name,
            RenderGenericNames {
                generics: self.generics,
                fields_filter: self.filter_generics.then_some(self.fields)
            },
            RenderConstructor(self.fields),
        )?;
        write!(f, "}}")
    }
}

struct RenderGenerics<'a> {
    generics: &'a [&'a str],
    fields_filter: Option<&'a Fields<'a>>,
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

        f.write_char('[')?;
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
            write!(f, "{value} any")?;
        }
        f.write_char(']')
    }
}

pub(super) struct RenderGenericNames<'a> {
    pub(super) generics: &'a [&'a str],
    pub(super) fields_filter: Option<&'a Fields<'a>>,
}

impl Display for RenderGenericNames<'_> {
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

        f.write_char('[')?;
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
        f.write_char(']')
    }
}

struct RenderGenericTypes<'a>(&'a [Type<'a>]);

impl Display for RenderGenericTypes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        f.write_char('[')?;
        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", RenderType(value))?;
        }
        f.write_char(']')
    }
}

struct RenderFields<'a>(&'a Fields<'a>);

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.fields.is_empty() {
            write!(f, "struct{{}}")
        } else {
            writeln!(f, "struct {{")?;

            for field in &*self.0.fields {
                writeln!(
                    f,
                    "{}\t{} {}",
                    RenderComment {
                        indent: Indent(1),
                        comment: &field.comment
                    },
                    heck::AsUpperCamelCase(&field.name),
                    RenderType(&field.ty)
                )?;
            }

            write!(f, "}}")
        }
    }
}

struct RenderParameters<'a>(&'a Fields<'a>);

impl Display for RenderParameters<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.fields.is_empty() {
            writeln!(f)?;
        }

        for field in &*self.0.fields {
            writeln!(
                f,
                "\t{} {},",
                heck::AsLowerCamelCase(&field.name),
                RenderType(&field.ty),
            )?;
        }

        Ok(())
    }
}

struct RenderConstructor<'a>(&'a Fields<'a>);

impl Display for RenderConstructor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.fields.is_empty() {
            write!(f, "{{}}")
        } else {
            writeln!(f, "{{")?;

            for field in &*self.0.fields {
                writeln!(
                    f,
                    "\t\t{}: {},",
                    heck::AsUpperCamelCase(&field.name),
                    heck::AsLowerCamelCase(&field.name)
                )?;
            }

            write!(f, "\t}}")
        }
    }
}

struct RenderAlias<'a>(&'a TypeAlias<'a>);

impl Display for RenderAlias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}type {} {}",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment,
            },
            heck::AsUpperCamelCase(&self.0.name),
            RenderType(&self.0.target),
        )
    }
}

struct RenderComment<'a> {
    indent: Indent,
    comment: &'a [&'a str],
}

impl Display for RenderComment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, comment } = *self;
        for line in comment {
            writeln!(f, "{indent}// {line}")?;
        }

        Ok(())
    }
}

pub(super) struct RenderType<'a>(pub(super) &'a Type<'a>);

impl Display for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "bool"),
            Type::U8 => write!(f, "uint8"),
            Type::U16 => write!(f, "uint16"),
            Type::U32 => write!(f, "uint32"),
            Type::U64 => write!(f, "uint64"),
            Type::U128 | Type::I128 => write!(f, "*big.Int"),
            Type::I8 => write!(f, "int8"),
            Type::I16 => write!(f, "int16"),
            Type::I32 => write!(f, "int32"),
            Type::I64 => write!(f, "int64"),
            Type::F32 => write!(f, "float32"),
            Type::F64 => write!(f, "float64"),
            Type::String | Type::StringRef | Type::BoxString => write!(f, "string"),
            Type::Bytes | Type::BytesRef | Type::BoxBytes => write!(f, "[]byte"),
            Type::Vec(ty) => write!(f, "[]{}", RenderType(ty)),
            Type::HashMap(kv) => write!(f, "map[{}]{}", RenderType(&kv.0), RenderType(&kv.1)),
            Type::HashSet(ty) => write!(f, "map[{}]struct{{}}", RenderType(ty)),
            Type::Option(ty) => write!(f, "*{}", RenderType(ty)),
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "mabo.NonZeroU8"),
                Type::U16 => write!(f, "mabo.NonZeroU16"),
                Type::U32 => write!(f, "mabo.NonZeroU32"),
                Type::U64 => write!(f, "mabo.NonZeroU64"),
                Type::U128 => write!(f, "mabo.NonZeroU128"),
                Type::I8 => write!(f, "mabo.NonZeroI8"),
                Type::I16 => write!(f, "mabo.NonZeroI16"),
                Type::I32 => write!(f, "mabo.NonZeroI32"),
                Type::I64 => write!(f, "mabo.NonZeroI64"),
                Type::I128 => write!(f, "mabo.NonZeroI128"),
                Type::F32 => write!(f, "mabo.NonZeroF32"),
                Type::F64 => write!(f, "mabo.NonZeroF64"),
                Type::String | Type::StringRef => write!(f, "mabo.NonZeroString"),
                Type::Bytes | Type::BytesRef => write!(f, "mabo.NonZeroBytes"),
                Type::Vec(ty) => write!(f, "mabo.NonZeroVec[{}]", RenderType(ty)),
                Type::HashMap(kv) => write!(
                    f,
                    "mabo.NonZeroHashMap[{}, {}]",
                    RenderType(&kv.0),
                    RenderType(&kv.1)
                ),
                Type::HashSet(ty) => write!(f, "mabo.NonZeroHashSet[{}]", RenderType(ty)),
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => write!(f, "mabo.Tuple{}{}", types.len(), Concat(types)),
            Type::Array(ty, size) => write!(f, "[{size}]{}", RenderType(ty)),
            Type::External(ExternalType {
                path,
                name,
                generics,
            }) => {
                if let Some(path) = path.last() {
                    write!(
                        f,
                        "{path}.{}{}",
                        heck::AsUpperCamelCase(name),
                        RenderGenericTypes(generics),
                    )
                } else {
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
}

struct RenderConstType<'a>(&'a Type<'a>);

impl Display for RenderConstType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "bool"),
            Type::U8 => write!(f, "uint8"),
            Type::U16 => write!(f, "uint16"),
            Type::U32 => write!(f, "uint32"),
            Type::U64 => write!(f, "uint64"),
            Type::U128 | Type::I128 => write!(f, "*big.Int"),
            Type::I8 => write!(f, "int8"),
            Type::I16 => write!(f, "int16"),
            Type::I32 => write!(f, "int32"),
            Type::I64 => write!(f, "int64"),
            Type::F32 => write!(f, "float32"),
            Type::F64 => write!(f, "float64"),
            Type::String | Type::StringRef => write!(f, "string"),
            Type::Bytes | Type::BytesRef => write!(f, "[]byte"),
            _ => panic!("invalid data type for const"),
        }
    }
}

struct Concat<'a>(&'a [Type<'a>]);

impl Display for Concat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        f.write_char('[')?;
        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            RenderType(value).fmt(f)?;
        }
        f.write_char(']')
    }
}

struct RenderConst<'a>(&'a Const<'a>);

impl Display for RenderConst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if matches!(
            self.0.ty,
            Type::Bool
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::F32
                | Type::F64
                | Type::String
                | Type::StringRef
        ) {
            "const"
        } else {
            "var"
        };

        writeln!(
            f,
            "{}{kind} {} {} = {}",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment
            },
            heck::AsUpperCamelCase(&self.0.name),
            RenderConstType(&self.0.ty),
            RenderLiteral(&self.0.value),
        )
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
                if b.is_empty() {
                    return Ok(());
                }

                f.write_str("[]byte{")?;
                for (i, value) in b.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    value.fmt(f)?;
                }
                f.write_char('}')
            }
        }
    }
}

struct RenderEnum<'a>(&'a Enum<'a>);

impl Display for RenderEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "type {}Variant interface {{",
            heck::AsUpperCamelCase(&self.0.name),
        )?;
        writeln!(f, "\t sealed()")?;
        writeln!(f, "}}")?;

        writeln!(
            f,
            "\n{}type {} {1}Variant",
            RenderComment {
                indent: Indent(0),
                comment: &self.0.comment
            },
            heck::AsUpperCamelCase(&self.0.name),
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

        Ok(())
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
            "{}type {}_{}{} {}",
            RenderComment {
                indent: Indent(0),
                comment: &self.variant.comment
            },
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenerics {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields)
            },
            RenderFields(&self.variant.fields)
        )?;

        writeln!(
            f,
            "\nfunc (v {}_{}{}) sealed() {{}}",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;

        writeln!(
            f,
            "\n{}",
            RenderNewFunc {
                name: format_args!(
                    "{}_{}",
                    heck::AsUpperCamelCase(self.enum_name),
                    heck::AsUpperCamelCase(&self.variant.name)
                ),
                generics: self.generics,
                fields: &self.variant.fields,
                filter_generics: true,
            },
        )?;

        write!(
            f,
            "\n{}\n{}\n{}",
            encode::RenderEnumVariant {
                enum_name: self.enum_name,
                generics: self.generics,
                variant: self.variant,
            },
            decode::RenderEnumVariant {
                enum_name: self.enum_name,
                generics: self.generics,
                variant: self.variant,
            },
            size::RenderEnumVariant {
                enum_name: self.enum_name,
                generics: self.generics,
                variant: self.variant,
            },
        )
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
