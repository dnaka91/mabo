#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{Enum, FieldKind, Fields, Struct, Type, Variant};

use crate::Indent;

pub(super) struct RenderStruct<'a> {
    pub indent: Indent,
    pub item: &'a Struct<'a>,
}

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;
        writeln!(f, "{indent}override fun encode(w: ByteBuffer) {{")?;
        writeln!(
            f,
            "{}{indent}}}",
            RenderFields {
                indent: indent + 1,
                item: &item.fields
            },
        )
    }
}

pub(super) struct RenderEnum<'a> {
    pub indent: Indent,
    pub item: &'a Enum<'a>,
}

impl Display for RenderEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;

        writeln!(f, "{indent}override fun encode(w: ByteBuffer) {{")?;
        writeln!(f, "{}when (this) {{", indent + 1)?;

        for variant in &item.variants {
            writeln!(
                f,
                "{}is {} -> Encoder.encodeVariantId(w, VariantId({}u))",
                indent + 2,
                heck::AsUpperCamelCase(variant.name),
                variant.id,
            )?;
        }

        writeln!(f, "{}}}", indent + 1)?;
        writeln!(f, "{indent}}}")
    }
}

pub(super) struct RenderEnumVariant<'a> {
    pub indent: Indent,
    pub item: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;
        writeln!(f, "{indent}override fun encode(w: ByteBuffer) {{")?;
        writeln!(f, "{}super.encode(w)", indent + 1)?;
        writeln!(
            f,
            "{}{indent}}}",
            RenderFields {
                indent: indent + 1,
                item: &item.fields
            },
        )
    }
}

struct RenderFields<'a> {
    indent: Indent,
    item: &'a Fields<'a>,
}

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.item.kind == FieldKind::Unit {
            return Ok(());
        }

        let Self { indent, item } = *self;

        for field in &*item.fields {
            if let Type::Option(ty) = &field.ty {
                writeln!(
                    f,
                    "{indent}Encoder.encodeFieldOption(w, {}, this.{}) {{ w, v -> {} }}",
                    field.id,
                    heck::AsLowerCamelCase(&field.name),
                    RenderType {
                        indent: indent + 1,
                        ty,
                        name: "v",
                    },
                )?;
            } else {
                writeln!(
                    f,
                    "{indent}Encoder.encodeField(w, {}) {{ w -> {} }}",
                    field.id,
                    RenderType {
                        indent: indent + 1,
                        ty: &field.ty,
                        name: format_args!("this.{}", heck::AsLowerCamelCase(&field.name)),
                    },
                )?;
            }
        }

        writeln!(f, "{indent}Encoder.encodeU32(w, END_MARKER)")
    }
}

struct RenderType<'a, T> {
    indent: Indent,
    ty: &'a Type<'a>,
    name: T,
}

impl<T> Display for RenderType<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            Type::Bool => write!(f, "Encoder.encodeBool(w, {})", self.name),
            Type::U8 => write!(f, "Encoder.encodeU8(w, {})", self.name),
            Type::U16 => write!(f, "Encoder.encodeU16(w, {})", self.name),
            Type::U32 => write!(f, "Encoder.encodeU32(w, {})", self.name),
            Type::U64 => write!(f, "Encoder.encodeU64(w, {})", self.name),
            Type::U128 => write!(f, "Encoder.encodeU128(w, {})", self.name),
            Type::I8 => write!(f, "Encoder.encodeI8(w, {})", self.name),
            Type::I16 => write!(f, "Encoder.encodeI16(w, {})", self.name),
            Type::I32 => write!(f, "Encoder.encodeI32(w, {})", self.name),
            Type::I64 => write!(f, "Encoder.encodeI64(w, {})", self.name),
            Type::I128 => write!(f, "Encoder.encodeI128(w, {})", self.name),
            Type::F32 => write!(f, "Encoder.encodeF32(w, {})", self.name),
            Type::F64 => write!(f, "Encoder.encodeF64(w, {})", self.name),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "Encoder.encodeString(w, {})", self.name)
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "Encoder.encodeBytes(w, {})", self.name)
            }
            Type::Vec(ty) => {
                writeln!(f, "Encoder.encodeVec(w, {}) {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{indent}{}",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1,
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{indent}}}", indent = self.indent)
            }
            Type::HashMap(kv) => {
                writeln!(f, "Encoder.encodeHashMap(")?;
                writeln!(f, "{indent}w, {},", self.name, indent = self.indent + 1,)?;

                writeln!(f, "{indent}{{ w, k ->", indent = self.indent + 1,)?;
                writeln!(
                    f,
                    "{indent}{}",
                    RenderType {
                        ty: &kv.0,
                        name: "k",
                        indent: self.indent + 2,
                    },
                    indent = self.indent + 2,
                )?;
                writeln!(f, "{indent}}},", indent = self.indent + 1)?;

                writeln!(f, "{indent}{{ w, v ->", indent = self.indent + 1,)?;
                writeln!(
                    f,
                    "{indent}{}",
                    RenderType {
                        ty: &kv.1,
                        name: "v",
                        indent: self.indent + 2,
                    },
                    indent = self.indent + 2,
                )?;
                writeln!(f, "{indent}}},", indent = self.indent + 1)?;
                write!(f, "{indent})", indent = self.indent)
            }
            Type::HashSet(ty) => {
                writeln!(f, "Encoder.encodeHashSet(w, {}) {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{indent}{}",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1,
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{indent}}}", indent = self.indent)
            }
            Type::Option(ty) => {
                writeln!(f, "Encoder.encodeOption(w, {}) {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{indent}{}",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1,
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{indent}}}", indent = self.indent)
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "Encoder.encodeU8(w, {}.get())", self.name),
                Type::U16 => write!(f, "Encoder.encodeU16(w, {}.get())", self.name),
                Type::U32 => write!(f, "Encoder.encodeU32(w, {}.get())", self.name),
                Type::U64 => write!(f, "Encoder.encodeU64(w, {}.get())", self.name),
                Type::U128 => write!(f, "Encoder.encodeU128(w, {}.get())", self.name),
                Type::I8 => write!(f, "Encoder.encodeI8(w, {}.get())", self.name),
                Type::I16 => write!(f, "Encoder.encodeI16(w, {}.get())", self.name),
                Type::I32 => write!(f, "Encoder.encodeI32(w, {}.get())", self.name),
                Type::I64 => write!(f, "Encoder.encodeI64(w, {}.get())", self.name),
                Type::I128 => write!(f, "Encoder.encodeI128(w, {}.get())", self.name),
                Type::String
                | Type::StringRef
                | Type::Bytes
                | Type::BytesRef
                | Type::Vec(_)
                | Type::HashMap(_)
                | Type::HashSet(_) => write!(
                    f,
                    "{}",
                    RenderType {
                        ty,
                        name: format_args!("{}.get()", self.name),
                        indent: self.indent,
                    }
                ),
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => match types.len() {
                2..=12 => {
                    writeln!(f, "w.let {{ w ->")?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{indent}{}",
                            RenderType {
                                ty,
                                name: format_args!(
                                    "{}.{}",
                                    self.name,
                                    match idx + 1 {
                                        1 => "first",
                                        2 => "second",
                                        3 => "third",
                                        4 => "fourth",
                                        5 => "fifth",
                                        6 => "sixth",
                                        7 => "seventh",
                                        8 => "eighth",
                                        9 => "ninth",
                                        10 => "tenth",
                                        11 => "eleventh",
                                        12 => "twelfth",
                                        _ => unreachable!(),
                                    }
                                ),
                                indent: self.indent + 1,
                            },
                            indent = self.indent + 1,
                        )?;
                    }
                    write!(f, "{indent}}}", indent = self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(f, "Enoder.encodeArray(w, {}) {{ w, v ->", self.name)?;
                    writeln!(
                        f,
                        "{indent}{}",
                        RenderType {
                            ty,
                            name: "v",
                            indent: self.indent + 1,
                        },
                        indent = self.indent + 1,
                    )?;
                    write!(f, "{indent}}}", indent = self.indent)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => write!(f, "{}.encode(w)", self.name),
        }
    }
}
