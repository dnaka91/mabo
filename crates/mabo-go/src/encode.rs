#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{FieldKind, Fields, Struct, Type, Variant};

use crate::{
    definition::{self, RenderGenericNames},
    Indent,
};

pub(super) struct RenderStruct<'a>(pub(super) &'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "var _ buf.Encode = (*{}{})(nil)\n",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;

        writeln!(
            f,
            "func (v *{}{}) Encode(w []byte) []byte {{",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;
        writeln!(f, "{}\treturn w\n}}", RenderFields(&self.0.fields))
    }
}

pub(super) struct RenderEnumVariant<'a> {
    pub(super) enum_name: &'a str,
    pub(super) generics: &'a [&'a str],
    pub(super) variant: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "var _ buf.Encode = (*{}_{}{})(nil)\n",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;

        writeln!(
            f,
            "func (v *{}_{}{}) Encode(w []byte) []byte {{",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;
        writeln!(f, "{}\treturn nil\n}}", RenderFields(&self.variant.fields))
    }
}

struct RenderFields<'a>(&'a Fields<'a>);

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.kind == FieldKind::Unit {
            return Ok(());
        }

        for field in &*self.0.fields {
            if let Type::Option(ty) = &field.ty {
                writeln!(
                    f,
                    "\tw = buf.EncodeFieldOption[{}](w, {}, &v.{}, func (w []byte, v {0}) []byte \
                     {{\n\t\treturn {}\n\t}})",
                    definition::RenderType(ty),
                    field.id,
                    heck::AsUpperCamelCase(&field.name),
                    RenderType {
                        ty,
                        name: "v",
                        indent: Indent(2),
                    },
                )?;
            } else {
                writeln!(
                    f,
                    "\tw = buf.EncodeField(w, {}, func (w []byte) []byte {{\n\t\treturn {}\n\t}})",
                    field.id,
                    RenderType {
                        ty: &field.ty,
                        name: format_args!("v.{}", heck::AsUpperCamelCase(&field.name)),
                        indent: Indent(2),
                    },
                )?;
            }
        }

        writeln!(f, "\tw = buf.EncodeU32(w, buf.EndMarker)")
    }
}

struct RenderType<'a, T> {
    ty: &'a Type<'a>,
    name: T,
    indent: Indent,
}

impl<T> Display for RenderType<'_, T>
where
    T: Copy + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ty, name, indent } = *self;
        match ty {
            Type::Bool => write!(f, "buf.EncodeBool(w, {name})"),
            Type::U8 => write!(f, "buf.EncodeU8(w, {name})"),
            Type::U16 => write!(f, "buf.EncodeU16(w, {name})"),
            Type::U32 => write!(f, "buf.EncodeU32(w, {name})"),
            Type::U64 => write!(f, "buf.EncodeU64(w, {name})"),
            Type::U128 => write!(f, "buf.EncodeU128(w, {name})"),
            Type::I8 => write!(f, "buf.EncodeI8(w, {name})"),
            Type::I16 => write!(f, "buf.EncodeI16(w, {name})"),
            Type::I32 => write!(f, "buf.EncodeI32(w, {name})"),
            Type::I64 => write!(f, "buf.EncodeI64(w, {name})"),
            Type::I128 => write!(f, "buf.EncodeI128(w, {name})"),
            Type::F32 => write!(f, "buf.EncodeF32(w, {name})"),
            Type::F64 => write!(f, "buf.EncodeF64(w, {name})"),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "buf.EncodeString(w, {name})")
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "buf.EncodeBytes(w, {name})")
            }
            Type::Vec(ty) => {
                writeln!(
                    f,
                    "buf.EncodeVec[{}](w, {name}, func(w []byte, v {0}) []byte {{",
                    definition::RenderType(ty),
                )?;
                writeln!(
                    f,
                    "{}return {}",
                    indent + 1,
                    RenderType {
                        ty,
                        name: "v",
                        indent: indent + 1,
                    },
                )?;
                write!(f, "{indent}}})")
            }
            Type::HashMap(kv) => {
                writeln!(
                    f,
                    "buf.EncodeHashMap[{}, {}](",
                    definition::RenderType(&kv.0),
                    definition::RenderType(&kv.1)
                )?;
                writeln!(f, "{}w, {name},", indent + 1)?;

                writeln!(
                    f,
                    "{}func(w []byte, k {}) []byte {{",
                    indent + 1,
                    definition::RenderType(&kv.0),
                )?;
                writeln!(
                    f,
                    "{}return {}",
                    indent + 2,
                    RenderType {
                        ty: &kv.0,
                        name: "k",
                        indent: indent + 2,
                    },
                )?;
                writeln!(f, "{}}},", indent + 1)?;

                writeln!(
                    f,
                    "{}func(w []byte, v {}) []byte {{",
                    indent + 1,
                    definition::RenderType(&kv.1),
                )?;
                writeln!(
                    f,
                    "{}return {}",
                    indent + 2,
                    RenderType {
                        ty: &kv.1,
                        name: "v",
                        indent: indent + 2,
                    },
                )?;
                writeln!(f, "{}}},", indent + 1)?;
                write!(f, "{indent})")
            }
            Type::HashSet(ty) => {
                writeln!(
                    f,
                    "buf.EncodeHashSet[{}](w, {name}, func(w []byte, v {0}) []byte {{",
                    definition::RenderType(ty),
                )?;
                writeln!(
                    f,
                    "{}return {}",
                    indent + 1,
                    RenderType {
                        ty,
                        name: "v",
                        indent: indent + 1,
                    },
                )?;
                write!(f, "{indent}}})")
            }
            Type::Option(ty) => {
                writeln!(
                    f,
                    "buf.EncodeOption[{}](w, {name}, func(w []byte, v {0}) []byte {{",
                    definition::RenderType(ty),
                )?;
                writeln!(
                    f,
                    "{}return {}",
                    indent + 1,
                    RenderType {
                        ty,
                        name: "v",
                        indent: indent + 1,
                    },
                )?;
                write!(f, "{indent}}})")
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "buf.EncodeU8(w, {name}.Get())"),
                Type::U16 => write!(f, "buf.EncodeU16(w, {name}.Get())"),
                Type::U32 => write!(f, "buf.EncodeU32(w, {name}.Get())"),
                Type::U64 => write!(f, "buf.EncodeU64(w, {name}.Get())"),
                Type::U128 => write!(f, "buf.EncodeU128(w, {name}.Get())"),
                Type::I8 => write!(f, "buf.EncodeI8(w, {name}.Get())"),
                Type::I16 => write!(f, "buf.EncodeI16(w, {name}.Get())"),
                Type::I32 => write!(f, "buf.EncodeI32(w, {name}.Get())"),
                Type::I64 => write!(f, "buf.EncodeI64(w, {name}.Get())"),
                Type::I128 => write!(f, "buf.EncodeI128(w, {name}.Get())"),
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
                        name: format_args!("{name}.Get()"),
                        indent,
                    }
                ),
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => match types.len() {
                2..=12 => {
                    writeln!(f, "func (w []byte) []byte {{")?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{}w = {}",
                            indent + 1,
                            RenderType {
                                ty,
                                name: format_args!("{name}.F{idx}"),
                                indent: indent + 1,
                            },
                        )?;
                    }
                    writeln!(f, "{}return w", indent + 1)?;
                    write!(f, "{indent}}}(w)")
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(
                        f,
                        "buf.EncodeArray{size}[{}](w, {name}, func(w []byte, v {0}) []byte {{",
                        definition::RenderType(ty),
                    )?;
                    writeln!(
                        f,
                        "{}return {}",
                        indent + 1,
                        RenderType {
                            ty,
                            name: "v",
                            indent: indent + 1,
                        },
                    )?;
                    write!(f, "{indent}}})")
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => write!(f, "{name}.Encode(w)"),
        }
    }
}
