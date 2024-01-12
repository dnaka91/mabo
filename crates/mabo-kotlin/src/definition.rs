use std::fmt::{self, Display, Write};

use mabo_compiler::simplify::{
    Const, Definition, Enum, ExternalType, FieldKind, Fields, Literal, Schema, Struct, Type,
    TypeAlias, Variant,
};

use crate::{decode, encode, size, Indent, Opts, Output};

/// Take a single schema and convert it into Kotlin source code (which can result in multiple
/// files).
#[must_use]
pub fn render_schema<'a>(
    opts: &'a Opts<'_>,
    Schema { definitions, .. }: &'a Schema<'_>,
) -> Output<'a> {
    let mut content = format!("{}{}", RenderPackage(opts.package, None), RenderImports,);

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
                "{}{}",
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
        }
        Definition::Enum(e) => writeln!(buf, "{}", RenderEnum(e)).unwrap(),
        Definition::TypeAlias(a) => writeln!(buf, "{}", RenderAlias(a)).unwrap(),
        Definition::Const(c) => writeln!(buf, "{}", RenderConst(c)).unwrap(),
        Definition::Import(_) => {}
    }

    None
}

struct RenderPackage<'a>(&'a str, Option<&'a [&'a str]>);

impl Display for RenderPackage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = self.1 {
            write!(f, "{}", RenderComment { indent: 0, comment })?;
        }

        writeln!(f, "package {}\n", heck::AsSnakeCase(self.0))
    }
}

struct RenderImports;

impl Display for RenderImports {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "import java.math.BigInteger")?;
        writeln!(f, "import java.nio.ByteBuffer")?;
        writeln!(f, "import kotlin.UByte")?;
        writeln!(f, "import kotlin.UShort")?;
        writeln!(f, "import kotlin.UInt")?;
        writeln!(f, "import kotlin.ULong")?;
        writeln!(f)?;
        writeln!(f, "import rocks.dnaka91.mabo.*")?;
        writeln!(f, "import rocks.dnaka91.mabo.buf.*")?;
        writeln!(f)
    }
}

struct RenderStruct<'a>(&'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.fields.kind == FieldKind::Unit {
            writeln!(
                f,
                "{}public object {} : Encode, Size {{",
                RenderComment {
                    indent: 0,
                    comment: &self.0.comment
                },
                heck::AsUpperCamelCase(&self.0.name),
            )?;
        } else {
            writeln!(
                f,
                "{}public data class {}{}(\n{}) : Encode, Size {{",
                RenderComment {
                    indent: 0,
                    comment: &self.0.comment
                },
                heck::AsUpperCamelCase(&self.0.name),
                RenderGenerics {
                    generics: &self.0.generics,
                    fields_filter: None
                },
                RenderFields {
                    indent: 0,
                    fields: &self.0.fields
                }
            )?;
        }

        writeln!(
            f,
            "{}\n{}\n",
            encode::RenderStruct {
                indent: Indent(1),
                item: self.0,
            },
            size::RenderStruct {
                indent: Indent(1),
                item: self.0,
            }
        )?;

        writeln!(
            f,
            "    companion object : Decode<{}{}> {{",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenerics {
                generics: &self.0.generics,
                fields_filter: None
            },
        )?;
        write!(
            f,
            "{}",
            decode::RenderStruct {
                indent: Indent(2),
                item: self.0,
            }
        )?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")
    }
}

struct RenderEnum<'a>(&'a Enum<'a>);

impl Display for RenderEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "public sealed interface {} {{",
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
        if self.variant.fields.kind == FieldKind::Unit {
            writeln!(
                f,
                "{}    public object {} : {}, Decode, Encode, Size {{",
                RenderComment {
                    indent: 1,
                    comment: &self.variant.comment
                },
                heck::AsUpperCamelCase(&self.variant.name),
                heck::AsUpperCamelCase(&self.enum_name),
            )?;
        } else {
            writeln!(
                f,
                "{}    public data class {}{}(\n{}    ) : {}, Decode, Encode, Size {{",
                RenderComment {
                    indent: 1,
                    comment: &self.variant.comment
                },
                heck::AsUpperCamelCase(&self.variant.name),
                RenderGenerics {
                    generics: self.generics,
                    fields_filter: Some(&self.variant.fields)
                },
                RenderFields {
                    indent: 1,
                    fields: &self.variant.fields
                },
                heck::AsUpperCamelCase(&self.enum_name),
            )?;
        }

        writeln!(
            f,
            "{}\n{:indent$}}}",
            size::RenderEnumVariant {
                indent: Indent(2),
                item: self.variant,
            },
            "",
            indent = 4,
        )
    }
}

struct RenderFields<'a> {
    indent: usize,
    fields: &'a Fields<'a>,
}

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.fields.fields.is_empty() {
            return Ok(());
        }

        let width = (self.indent + 1) * 4;
        for field in &*self.fields.fields {
            writeln!(
                f,
                "{}{:width$}val {}: {},",
                RenderComment {
                    indent: self.indent + 1,
                    comment: &field.comment
                },
                "",
                heck::AsLowerCamelCase(&field.name),
                RenderType(&field.ty)
            )?;
        }
        Ok(())
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
                if !self.generics.iter().any(|gen| uses_generic(gen, fields)) {
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
            .filter(|gen| match self.fields_filter {
                Some(fields) => uses_generic(gen, fields),
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
            "{}typealias {}{} = {}",
            RenderComment {
                indent: 0,
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
            "{}const val {}: {} = {}",
            RenderComment {
                indent: 0,
                comment: &self.0.comment
            },
            heck::AsShoutySnakeCase(&self.0.name),
            RenderConstType(&self.0.ty),
            RenderLiteral {
                value: &self.0.value,
                unsigned: matches!(
                    self.0.ty,
                    Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                )
            }
        )
    }
}

struct RenderConstType<'a>(&'a Type<'a>);

impl Display for RenderConstType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "Boolean"),
            Type::U8 => write!(f, "UByte"),
            Type::U16 => write!(f, "UShort"),
            Type::U32 => write!(f, "UInt"),
            Type::U64 => write!(f, "ULong"),
            Type::U128 | Type::I128 => write!(f, "BigInteger"),
            Type::I8 => write!(f, "Byte"),
            Type::I16 => write!(f, "Short"),
            Type::I32 => write!(f, "Int"),
            Type::I64 => write!(f, "Long"),
            Type::F32 => write!(f, "Float"),
            Type::F64 => write!(f, "Double"),
            Type::String | Type::StringRef => write!(f, "String"),
            Type::Bytes | Type::BytesRef => write!(f, "ByteArray"),
            _ => panic!("invalid data type for const"),
        }
    }
}

struct RenderLiteral<'a> {
    value: &'a Literal,
    unsigned: bool,
}

impl Display for RenderLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Int(i) => write!(f, "{i}{}", if self.unsigned { "u" } else { "" }),
            Literal::Float(f2) => write!(f, "{f2}"),
            Literal::String(s) => write!(f, "{s:?}"),
            Literal::Bytes(b) => {
                if b.is_empty() {
                    return Ok(());
                }

                f.write_str("byteArrayOf(")?;
                for (i, value) in b.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    value.fmt(f)?;
                }
                f.write_char(')')
            }
        }
    }
}

struct RenderComment<'a> {
    indent: usize,
    comment: &'a [&'a str],
}

impl Display for RenderComment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.comment.is_empty() {
            return Ok(());
        }

        let width = self.indent * 4;
        writeln!(f, "{:width$}/**", "")?;
        for line in self.comment {
            writeln!(f, "{:width$} * {}", "", line)?;
        }
        writeln!(f, "{:width$} */", "")
    }
}

pub(super) struct RenderType<'a>(pub(super) &'a Type<'a>);

impl Display for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Bool => write!(f, "Boolean"),
            Type::U8 => write!(f, "UByte"),
            Type::U16 => write!(f, "UShort"),
            Type::U32 => write!(f, "UInt"),
            Type::U64 => write!(f, "ULong"),
            Type::U128 | Type::I128 => write!(f, "BigInteger"),
            Type::I8 => write!(f, "Byte"),
            Type::I16 => write!(f, "Short"),
            Type::I32 => write!(f, "Int"),
            Type::I64 => write!(f, "Long"),
            Type::F32 => write!(f, "Float"),
            Type::F64 => write!(f, "Double"),
            Type::String | Type::StringRef | Type::BoxString => write!(f, "String"),
            Type::Bytes | Type::BytesRef | Type::BoxBytes => write!(f, "ByteArray"),
            Type::Vec(ty) => write!(f, "List<{}>", RenderType(ty)),
            Type::HashMap(kv) => write!(f, "Map<{}, {}>", RenderType(&kv.0), RenderType(&kv.1)),
            Type::HashSet(ty) => write!(f, "Set<{}>", RenderType(ty)),
            Type::Option(ty) => write!(f, "{}?", RenderType(ty)),
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "NonZeroNumber<UByte>"),
                Type::U16 => write!(f, "NonZeroNumber<UShort>"),
                Type::U32 => write!(f, "NonZeroNumber<UInt>"),
                Type::U64 => write!(f, "NonZeroNumber<ULong>"),
                Type::U128 => write!(f, "NonZeroNumber<BigInteger>"),
                Type::I8 => write!(f, "NonZeroNumber<Byte>"),
                Type::I16 => write!(f, "NonZeroNumber<Short>"),
                Type::I32 => write!(f, "NonZeroNumber<Int>"),
                Type::I64 => write!(f, "NonZeroNumber<Long>"),
                Type::I128 => write!(f, "NonZeroNumber<BigInteger"),
                Type::F32 => write!(f, "NonZeroNumber<Float>"),
                Type::F64 => write!(f, "NonZeroNumber<Double>"),
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
            Type::Tuple(types) => write!(f, "Tuple{}{}", types.len(), Concat(types)),
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

        f.write_char('<')?;
        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            RenderType(value).fmt(f)?;
        }
        f.write_char('>')
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
