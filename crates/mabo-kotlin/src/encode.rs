#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{FieldKind, Fields, Struct, Type, Variant};

pub(super) struct RenderStruct<'a> {
    pub indent: usize,
    pub item: &'a Struct<'a>,
}

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent * 4;
        writeln!(f, "{:indent$}override fun encode(w: ByteBuffer) {{", "")?;
        writeln!(
            f,
            "{}{:indent$}}}",
            RenderFields {
                indent: self.indent + 1,
                item: &self.item.fields
            },
            "",
        )
    }
}

pub(super) struct RenderEnumVariant<'a> {
    pub indent: usize,
    pub variant: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = self.indent * 4;
        writeln!(f, "{:indent$}override fun encode(w: ByteBuffer) {{", "")?;
        writeln!(
            f,
            "{}{:indent$}return nil{1:indent$}}}",
            RenderFields {
                indent: self.indent + 1,
                item: &self.variant.fields
            },
            ""
        )
    }
}

struct RenderFields<'a> {
    indent: usize,
    item: &'a Fields<'a>,
}

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.item.kind == FieldKind::Unit {
            return Ok(());
        }

        let indent = self.indent * 4;

        for field in &*self.item.fields {
            if let Type::Option(ty) = &field.ty {
                writeln!(
                    f,
                    "{:indent$}Encoder.encodeFieldOption(w, {}, this.{}, {{ w, v -> {} }})",
                    "",
                    field.id,
                    heck::AsUpperCamelCase(&field.name),
                    RenderType {
                        ty,
                        name: "v",
                        indent: 2,
                    },
                )?;
            } else {
                writeln!(
                    f,
                    "{:indent$}Encoder.encodeField(w, {}, {{ w -> {} }})",
                    "",
                    field.id,
                    RenderType {
                        ty: &field.ty,
                        name: format_args!("this.{}", heck::AsUpperCamelCase(&field.name)),
                        indent: 2,
                    },
                )?;
            }
        }

        writeln!(f, "{:indent$}Encoder.encodeU32(w, END_MARKER)", "")
    }
}

struct RenderType<'a, T> {
    ty: &'a Type<'a>,
    name: T,
    indent: usize,
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
                writeln!(f, "Encoder.encodeVec(w, {}, {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{:indent$}{}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = (self.indent + 1) * 4,
                )?;
                write!(f, "{:indent$}}})", "", indent = self.indent * 4)
            }
            Type::HashMap(kv) => {
                writeln!(f, "Encoder.encodeHashMap(")?;
                writeln!(
                    f,
                    "{:indent$}w, {},",
                    "",
                    self.name,
                    indent = (self.indent + 1) * 4
                )?;

                writeln!(
                    f,
                    "{:indent$}{{ w, k ->",
                    "",
                    indent = (self.indent + 1) * 4
                )?;
                writeln!(
                    f,
                    "{:indent$}{}",
                    "",
                    RenderType {
                        ty: &kv.0,
                        name: "k",
                        indent: self.indent + 2
                    },
                    indent = (self.indent + 2) * 4,
                )?;
                writeln!(f, "{:indent$}}},", "", indent = (self.indent + 1) * 4)?;

                writeln!(
                    f,
                    "{:indent$}{{ w, v ->",
                    "",
                    indent = (self.indent + 1) * 4
                )?;
                writeln!(
                    f,
                    "{:indent$}{}",
                    "",
                    RenderType {
                        ty: &kv.1,
                        name: "v",
                        indent: self.indent + 2
                    },
                    indent = (self.indent + 2) * 4,
                )?;
                writeln!(f, "{:indent$}}},", "", indent = (self.indent + 1) * 4)?;
                write!(f, "{:indent$})", "", indent = self.indent * 4)
            }
            Type::HashSet(ty) => {
                writeln!(f, "Encoder.encodeHashSet(w, {}, {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{:indent$}{}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = (self.indent + 1) * 4,
                )?;
                write!(f, "{:indent$}}})", "", indent = self.indent * 4)
            }
            Type::Option(ty) => {
                writeln!(f, "Encoder.encodeOption(w, {}, {{ w, v ->", self.name)?;
                writeln!(
                    f,
                    "{:indent$}{}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = (self.indent + 1) * 4,
                )?;
                write!(f, "{:indent$}}})", "", indent = self.indent * 4)
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
                            "{:indent$}{}",
                            "",
                            RenderType {
                                ty,
                                name: format_args!("{}.F{}", self.name, idx),
                                indent: self.indent + 1,
                            },
                            indent = (self.indent + 1) * 4,
                        )?;
                    }
                    write!(f, "{:indent$}}}", "", indent = self.indent * 4)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(f, "Enoder.encodeArray(w, {}, {{ w, v ->", self.name)?;
                    writeln!(
                        f,
                        "{:indent$}{}",
                        "",
                        RenderType {
                            ty,
                            name: "v",
                            indent: self.indent + 1
                        },
                        indent = (self.indent + 1) * 4,
                    )?;
                    write!(f, "{:indent$}}})", "", indent = self.indent * 4)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => write!(f, "{}.encode(w)", self.name),
        }
    }
}
