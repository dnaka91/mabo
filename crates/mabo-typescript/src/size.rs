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
        writeln!(f, "{indent}size(): number {{")?;
        writeln!(
            f,
            "{}return 0{};",
            indent + 1,
            RenderFields {
                indent: indent + 2,
                item: &item.fields
            },
        )?;
        write!(f, "{indent}}}")
    }
}

pub(super) struct RenderEnum<'a> {
    pub indent: Indent,
    pub item: &'a Enum<'a>,
}

impl Display for RenderEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;

        writeln!(f, "{indent}size(): number {{")?;

        for variant in &item.variants {
            writeln!(
                f,
                "{}is {} -> size.sizeVariantId({})",
                indent + 1,
                heck::AsUpperCamelCase(variant.name),
                variant.id
            )?;
        }

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
        writeln!(f, "{indent}size(): number {{")?;
        writeln!(
            f,
            "{}return super.size(){};",
            indent + 1,
            RenderFields {
                indent: indent + 2,
                item: &item.fields
            }
        )?;
        write!(f, "{indent}}}")
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
            writeln!(f, " +")?;

            if let Type::Option(ty) = &field.ty {
                write!(
                    f,
                    "{indent}size.sizeFieldOption({}, this.{}, (v) => {{ {} }})",
                    field.id,
                    heck::AsUpperCamelCase(&field.name),
                    RenderType {
                        indent: indent + 1,
                        ty,
                        name: "v",
                    },
                )?;
            } else {
                write!(
                    f,
                    "{indent}size.sizeField({}, () => {{ {} }})",
                    field.id,
                    RenderType {
                        indent: indent + 1,
                        ty: &field.ty,
                        name: format_args!("this.{}", heck::AsUpperCamelCase(&field.name)),
                    },
                )?;
            }
        }

        write!(f, " +\n{indent}size.sizeFieldId(END_MARKER)")
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
            Type::Bool => write!(f, "size.sizeBool({})", self.name),
            Type::U8 => write!(f, "size.sizeU8({})", self.name),
            Type::U16 => write!(f, "size.sizeU16({})", self.name),
            Type::U32 => write!(f, "size.sizeU32({})", self.name),
            Type::U64 => write!(f, "size.sizeU64({})", self.name),
            Type::U128 => write!(f, "size.sizeU128({})", self.name),
            Type::I8 => write!(f, "size.sizeI8({})", self.name),
            Type::I16 => write!(f, "size.sizeI16({})", self.name),
            Type::I32 => write!(f, "size.sizeI32({})", self.name),
            Type::I64 => write!(f, "size.sizeI64({})", self.name),
            Type::I128 => write!(f, "size.sizeI128({})", self.name),
            Type::F32 => write!(f, "size.sizeF32({})", self.name),
            Type::F64 => write!(f, "size.sizeF64({})", self.name),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "size.sizeString({})", self.name)
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "size.sizeBytes({})", self.name)
            }
            Type::Vec(ty) => {
                writeln!(f, "size.sizeVec({}, (v) => {{", self.name)?;
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
                write!(f, "{indent}}})", indent = self.indent)
            }
            Type::HashMap(kv) => {
                writeln!(f, "size.sizeHashMap(",)?;
                writeln!(f, "{indent}{},", self.name, indent = self.indent + 1,)?;

                writeln!(f, "{indent}(k) => {{", indent = self.indent + 1)?;
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

                writeln!(f, "{indent}(v) => {{", indent = self.indent + 1)?;
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
                writeln!(f, "size.sizeHashSet({}, (v) => {{", self.name)?;
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
                write!(f, "{indent}}})", indent = self.indent)
            }
            Type::Option(ty) => {
                writeln!(f, "size.sizeOption({}, (v) => {{", self.name)?;
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
                write!(f, "{indent}}})", indent = self.indent)
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "size.sizeU8({}.get())", self.name),
                Type::U16 => write!(f, "size.sizeU16({}.get())", self.name),
                Type::U32 => write!(f, "size.sizeU32({}.get())", self.name),
                Type::U64 => write!(f, "size.sizeU64({}.get())", self.name),
                Type::U128 => write!(f, "size.sizeU128({}.get())", self.name),
                Type::I8 => write!(f, "size.sizeI8({}.get())", self.name),
                Type::I16 => write!(f, "size.sizeI16({}.get())", self.name),
                Type::I32 => write!(f, "size.sizeI32({}.get())", self.name),
                Type::I64 => write!(f, "size.sizeI64({}.get())", self.name),
                Type::I128 => write!(f, "size.sizeI128({}.get())", self.name),
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
                    writeln!(f, "() => {{")?;
                    writeln!(f, "{indent}0", indent = self.indent + 1)?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{indent}+ {}",
                            RenderType {
                                ty,
                                name: format_args!("{}.F{}", self.name, idx),
                                indent: self.indent + 1,
                            },
                            indent = self.indent + 1,
                        )?;
                    }
                    write!(f, "{indent}}}()", indent = self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(f, "size.sizeArray({}, (v) => {{", self.name)?;
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
                    write!(f, "{indent}}})", indent = self.indent)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => write!(f, "{}.size()", self.name),
        }
    }
}
