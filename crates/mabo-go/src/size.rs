#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{FieldKind, Fields, Struct, Type, Variant};

use crate::definition::{self, RenderGenericNames};

pub(super) struct RenderStruct<'a>(pub(super) &'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "var _ buf.Size = (*{}{})(nil)\n",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;

        writeln!(
            f,
            "func (v *{}{}) Size() int {{",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;
        writeln!(f, "\tsize := 0")?;
        writeln!(f, "{}\treturn size\n}}", RenderFields(&self.0.fields))
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
            "var _ buf.Size = (*{}_{}{})(nil)\n",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;

        writeln!(
            f,
            "func (v *{}_{}{}) Size() int {{",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;
        writeln!(f, "\tsize := 0")?;
        writeln!(f, "{}\treturn size\n}}", RenderFields(&self.variant.fields))
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
                    "\tsize += buf.SizeFieldOption[{}]({}, &v.{}, func (v {0}) int {{\n\t\treturn \
                     {}\n\t}})",
                    definition::RenderType(ty),
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
                    "\tsize += buf.SizeField({}, func() int {{\n\t\treturn {}\n\t}})",
                    field.id,
                    RenderType {
                        ty: &field.ty,
                        name: format_args!("v.{}", heck::AsUpperCamelCase(&field.name)),
                        indent: 2,
                    },
                )?;
            }
        }

        writeln!(f, "\tsize += buf.SizeU32(buf.EndMarker)")
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
            Type::Bool => write!(f, "buf.SizeBool({})", self.name),
            Type::U8 => write!(f, "buf.SizeU8({})", self.name),
            Type::U16 => write!(f, "buf.SizeU16({})", self.name),
            Type::U32 => write!(f, "buf.SizeU32({})", self.name),
            Type::U64 => write!(f, "buf.SizeU64({})", self.name),
            Type::U128 => write!(f, "buf.SizeU128({})", self.name),
            Type::I8 => write!(f, "buf.SizeI8({})", self.name),
            Type::I16 => write!(f, "buf.SizeI16({})", self.name),
            Type::I32 => write!(f, "buf.SizeI32({})", self.name),
            Type::I64 => write!(f, "buf.SizeI64({})", self.name),
            Type::I128 => write!(f, "buf.SizeI128({})", self.name),
            Type::F32 => write!(f, "buf.SizeF32({})", self.name),
            Type::F64 => write!(f, "buf.SizeF64({})", self.name),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "buf.SizeString({})", self.name)
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "buf.SizeBytes({})", self.name)
            }
            Type::Vec(ty) => {
                writeln!(
                    f,
                    "buf.SizeVec[{}]({}, func(v {0}) int {{",
                    definition::RenderType(ty),
                    self.name
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return {}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{:\t<indent$}}})", "", indent = self.indent)
            }
            Type::HashMap(kv) => {
                writeln!(
                    f,
                    "buf.SizeHashMap[{}, {}](",
                    definition::RenderType(&kv.0),
                    definition::RenderType(&kv.1)
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}{},",
                    "",
                    self.name,
                    indent = self.indent + 1
                )?;

                writeln!(
                    f,
                    "{:\t<indent$}func(k {}) int {{",
                    "",
                    definition::RenderType(&kv.0),
                    indent = self.indent + 1
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return {}",
                    "",
                    RenderType {
                        ty: &kv.0,
                        name: "k",
                        indent: self.indent + 2
                    },
                    indent = self.indent + 2,
                )?;
                writeln!(f, "{:\t<indent$}}},", "", indent = self.indent + 1)?;

                writeln!(
                    f,
                    "{:\t<indent$}func(v {}) int {{",
                    "",
                    definition::RenderType(&kv.1),
                    indent = self.indent + 1
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return {}",
                    "",
                    RenderType {
                        ty: &kv.1,
                        name: "v",
                        indent: self.indent + 2
                    },
                    indent = self.indent + 2,
                )?;
                writeln!(f, "{:\t<indent$}}},", "", indent = self.indent + 1)?;
                write!(f, "{:\t<indent$})", "", indent = self.indent)
            }
            Type::HashSet(ty) => {
                writeln!(
                    f,
                    "buf.SizeHashSet[{}]({}, func(v {0}) int {{",
                    definition::RenderType(ty),
                    self.name
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return {}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{:\t<indent$}}})", "", indent = self.indent)
            }
            Type::Option(ty) => {
                writeln!(
                    f,
                    "buf.SizeOption[{}]({}, func(v {0}) int {{",
                    definition::RenderType(ty),
                    self.name
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return {}",
                    "",
                    RenderType {
                        ty,
                        name: "v",
                        indent: self.indent + 1
                    },
                    indent = self.indent + 1,
                )?;
                write!(f, "{:\t<indent$}}})", "", indent = self.indent)
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "buf.SizeU8({}.Get())", self.name),
                Type::U16 => write!(f, "buf.SizeU16({}.Get())", self.name),
                Type::U32 => write!(f, "buf.SizeU32({}.Get())", self.name),
                Type::U64 => write!(f, "buf.SizeU64({}.Get())", self.name),
                Type::U128 => write!(f, "buf.SizeU128({}.Get())", self.name),
                Type::I8 => write!(f, "buf.SizeI8({}.Get())", self.name),
                Type::I16 => write!(f, "buf.SizeI16({}.Get())", self.name),
                Type::I32 => write!(f, "buf.SizeI32({}.Get())", self.name),
                Type::I64 => write!(f, "buf.SizeI64({}.Get())", self.name),
                Type::I128 => write!(f, "buf.SizeI128({}.Get())", self.name),
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
                        name: format_args!("{}.Get()", self.name),
                        indent: self.indent,
                    }
                ),
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => match types.len() {
                2..=12 => {
                    writeln!(f, "func() int {{")?;
                    writeln!(f, "{:\t<indent$}size := 0", "", indent = self.indent + 1)?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{:\t<indent$}size += {}",
                            "",
                            RenderType {
                                ty,
                                name: format_args!("{}.F{}", self.name, idx),
                                indent: self.indent + 1,
                            },
                            indent = self.indent + 1,
                        )?;
                    }
                    writeln!(f, "{:\t<indent$}return size", "", indent = self.indent + 1)?;
                    write!(f, "{:\t<indent$}}}(size)", "", indent = self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(
                        f,
                        "buf.SizeArray{size}[{}]({}, func(v {0}) int {{",
                        definition::RenderType(ty),
                        self.name
                    )?;
                    writeln!(
                        f,
                        "{:\t<indent$}return {}",
                        "",
                        RenderType {
                            ty,
                            name: "v",
                            indent: self.indent + 1
                        },
                        indent = self.indent + 1,
                    )?;
                    write!(f, "{:\t<indent$}}})", "", indent = self.indent)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => write!(f, "{}.Size()", self.name),
        }
    }
}
