#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{FieldKind, Fields, Struct, Type, Variant};

use crate::{Indent, definition};

pub(super) struct RenderStruct<'a> {
    pub indent: Indent,
    pub item: &'a Struct<'a>,
}

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;

        if item.fields.kind == FieldKind::Unit {
            return writeln!(
                f,
                "{indent}override fun decode(r: ByteBuffer): Result<{}> = Result.success({0})",
                heck::AsUpperCamelCase(&item.name),
            );
        }

        let indent_p1 = indent + 1;
        let indent_p2 = indent + 2;
        let indent_p3 = indent + 3;

        writeln!(
            f,
            "{indent}override fun decode(r: ByteBuffer): Result<{}{}> = runCatching {{",
            heck::AsUpperCamelCase(item.name),
            definition::RenderGenerics {
                generics: &item.generics,
                fields_filter: None,
            }
        )?;
        writeln!(
            f,
            "{}",
            RenderFieldVars {
                indent: indent_p1,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p1}while (true) {{")?;
        writeln!(
            f,
            "{indent_p2}val id = Decoder.decodeFieldId(r).getOrThrow()"
        )?;
        writeln!(f, "{indent_p2}when (id.value) {{")?;
        write!(
            f,
            "{}",
            RenderFields {
                indent: indent_p3,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p2}}}")?;
        writeln!(f, "{indent_p1}}}")?;
        writeln!(f)?;
        writeln!(f, "{indent_p1}{}(", heck::AsUpperCamelCase(&item.name))?;
        write!(
            f,
            "{}",
            RenderFoundChecks {
                indent: indent_p2,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p1})")?;
        writeln!(f, "{indent}}}")
    }
}

pub(super) struct RenderEnumVariant<'a> {
    pub indent: Indent,
    pub generics: &'a [&'a str],
    pub item: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            indent,
            generics,
            item,
        } = *self;

        if item.fields.kind == FieldKind::Unit {
            return writeln!(
                f,
                "{indent}override fun decode(r: ByteBuffer): Result<{}> = Result.success({0})",
                heck::AsUpperCamelCase(&item.name),
            );
        }

        let indent_p1 = indent + 1;
        let indent_p2 = indent + 2;
        let indent_p3 = indent + 3;

        writeln!(
            f,
            "{indent}override fun decode(r: ByteBuffer) Result<{}{}> = runCatching {{",
            heck::AsUpperCamelCase(item.name),
            definition::RenderGenerics {
                generics,
                fields_filter: Some(&item.fields),
            }
        )?;
        writeln!(
            f,
            "{}",
            RenderFieldVars {
                indent: indent_p1,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p1}while (true) {{")?;
        writeln!(
            f,
            "{indent_p2}val id = Decoder.decodeFieldId(r).getOrThrow()"
        )?;
        writeln!(f, "{indent_p2}when (id.value) {{")?;
        write!(
            f,
            "{}",
            RenderFields {
                indent: indent_p3,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p2}}}")?;
        writeln!(f, "{indent_p1}}}")?;
        writeln!(f)?;
        writeln!(f, "{indent_p1}{}(", heck::AsUpperCamelCase(&item.name))?;
        write!(
            f,
            "{}",
            RenderFoundChecks {
                indent: indent_p2,
                item: &item.fields
            }
        )?;
        writeln!(f, "{indent_p1})")?;
        writeln!(f, "{indent}}}")
    }
}

struct RenderFieldVars<'a> {
    indent: Indent,
    item: &'a Fields<'a>,
}

impl Display for RenderFieldVars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = self;
        for field in &*item.fields {
            writeln!(
                f,
                "{indent}var {}: {}{} = null",
                heck::AsLowerCamelCase(&field.name),
                definition::RenderType(&field.ty),
                if matches!(field.ty, Type::Option(_)) {
                    ""
                } else {
                    "?"
                },
            )?;
        }

        Ok(())
    }
}

struct RenderFields<'a> {
    indent: Indent,
    item: &'a Fields<'a>,
}

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;
        for field in &*item.fields {
            writeln!(
                f,
                "{indent}{} -> {} = {}.getOrThrow()",
                field.id,
                heck::AsLowerCamelCase(&field.name),
                RenderType {
                    ty: &field.ty,
                    indent: indent + 1,
                }
            )?;
        }

        writeln!(f, "{indent}END_MARKER -> break")
    }
}

struct RenderFoundChecks<'a> {
    indent: Indent,
    item: &'a Fields<'a>,
}

impl Display for RenderFoundChecks<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { indent, item } = *self;

        for field in &*item.fields {
            if matches!(field.ty, Type::Option(_)) {
                writeln!(f, "{indent}{},", heck::AsLowerCamelCase(&field.name))?;
            } else {
                write!(
                    f,
                    "{indent}{} ?: throw MissingFieldException({}, ",
                    heck::AsLowerCamelCase(&field.name),
                    field.id,
                )?;

                if item.kind == FieldKind::Named {
                    write!(f, "\"{}\"", field.name)?;
                } else {
                    write!(f, "null")?;
                }

                writeln!(f, "),")?;
            }
        }

        Ok(())
    }
}

struct RenderType<'a> {
    ty: &'a Type<'a>,
    indent: Indent,
}

impl Display for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            Type::Bool => write!(f, "Decoder.decodeBool(r)"),
            Type::U8 => write!(f, "Decoder.decodeU8(r)"),
            Type::U16 => write!(f, "Decoder.decodeU16(r)"),
            Type::U32 => write!(f, "Decoder.decodeU32(r)"),
            Type::U64 => write!(f, "Decoder.decodeU64(r)"),
            Type::U128 => write!(f, "Decoder.decodeU128(r)"),
            Type::I8 => write!(f, "Decoder.decodeI8(r)"),
            Type::I16 => write!(f, "Decoder.decodeI16(r)"),
            Type::I32 => write!(f, "Decoder.decodeI32(r)"),
            Type::I64 => write!(f, "Decoder.decodeI64(r)"),
            Type::I128 => write!(f, "Decoder.decodeI128(r)"),
            Type::F32 => write!(f, "Decoder.decodeF32(r)"),
            Type::F64 => write!(f, "Decoder.decodeF64(r)"),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "Decoder.decodeString(r)")
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "Decoder.decodeBytes(r)")
            }
            Type::Vec(ty) => {
                write!(
                    f,
                    "{}",
                    DecodeGenericSingle {
                        name: "decodeVec",
                        ty,
                        indent: self.indent,
                    }
                )
            }
            Type::HashMap(kv) => {
                write!(
                    f,
                    "{}",
                    DecodeGenericPair {
                        name: "decodeHashMap",
                        pair: kv,
                        indent: self.indent
                    }
                )
            }
            Type::HashSet(ty) => {
                write!(
                    f,
                    "{}",
                    DecodeGenericSingle {
                        name: "decodeHashSet",
                        ty,
                        indent: self.indent,
                    }
                )
            }
            Type::Option(ty) => {
                write!(
                    f,
                    "{}",
                    DecodeGenericSingle {
                        name: "decodeOption",
                        ty,
                        indent: self.indent,
                    }
                )
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "Decoder.decodeNonZeroU8(r)"),
                Type::U16 => write!(f, "Decoder.decodeNonZeroU16(r)"),
                Type::U32 => write!(f, "Decoder.decodeNonZeroU32(r)"),
                Type::U64 => write!(f, "Decoder.decodeNonZeroU64(r)"),
                Type::U128 => write!(f, "Decoder.decodeNonZeroU128(r)"),
                Type::I8 => write!(f, "Decoder.decodeNonZeroI8(r)"),
                Type::I16 => write!(f, "Decoder.decodeNonZeroI16(r)"),
                Type::I32 => write!(f, "Decoder.decodeNonZeroI32(r)"),
                Type::I64 => write!(f, "Decoder.decodeNonZeroI64(r)"),
                Type::I128 => write!(f, "Decoder.decodeNonZeroI128(r)"),
                Type::String | Type::StringRef => write!(f, "Decoder.decodeNonZeroString(r)"),
                Type::Bytes | Type::BytesRef => write!(f, "Decoder.decodeNonZeroBytes(r)"),
                Type::Vec(ty) => {
                    write!(
                        f,
                        "{}",
                        DecodeGenericSingle {
                            name: "decodeNonZeroVec",
                            ty,
                            indent: self.indent,
                        }
                    )
                }
                Type::HashMap(kv) => {
                    write!(
                        f,
                        "{}",
                        DecodeGenericPair {
                            name: "decodeNonZeroHashMap",
                            pair: kv,
                            indent: self.indent
                        }
                    )
                }
                Type::HashSet(ty) => {
                    write!(
                        f,
                        "{}",
                        DecodeGenericSingle {
                            name: "decodeNonZeroHashSet",
                            ty,
                            indent: self.indent,
                        }
                    )
                }
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => match types.len() {
                n @ 2..=12 => {
                    writeln!(f, "runCatching {{")?;
                    writeln!(f, "{}Tuple{n}(", self.indent + 1)?;
                    for ty in &**types {
                        writeln!(
                            f,
                            "{}{}",
                            self.indent + 2,
                            RenderType {
                                ty,
                                indent: self.indent + 2
                            }
                        )?;
                    }
                    writeln!(f, "{})", self.indent + 1)?;
                    write!(f, "{}}}", self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, _) => {
                writeln!(f, "Decoder.DecodeArray(r) {{ r ->")?;
                writeln!(
                    f,
                    "{}{}",
                    self.indent + 1,
                    RenderType {
                        ty,
                        indent: self.indent + 1
                    },
                )?;
                write!(f, "{}}}", self.indent)
            }
            Type::External(_) => {
                write!(f, "{}.decode(r)", definition::RenderType(self.ty))
            }
        }
    }
}

struct DecodeGenericSingle<'a> {
    name: &'static str,
    ty: &'a Type<'a>,
    indent: Indent,
}

impl Display for DecodeGenericSingle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Decoder.{}(r) {{ r ->", self.name,)?;
        writeln!(
            f,
            "{}{}",
            self.indent + 1,
            RenderType {
                ty: self.ty,
                indent: self.indent + 1
            },
        )?;
        write!(f, "{}}}", self.indent)
    }
}

struct DecodeGenericPair<'a> {
    name: &'static str,
    pair: &'a (Type<'a>, Type<'a>),
    indent: Indent,
}

impl Display for DecodeGenericPair<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Decoder.{}(", self.name,)?;
        writeln!(f, "{}r,", self.indent + 1)?;

        writeln!(f, "{}{{ r ->", self.indent + 1,)?;
        writeln!(
            f,
            "{}{}",
            self.indent + 2,
            RenderType {
                ty: &self.pair.0,
                indent: self.indent + 2
            },
        )?;
        writeln!(f, "{}}},", self.indent + 1)?;

        writeln!(f, "{}{{ r ->", self.indent + 1,)?;
        writeln!(
            f,
            "{}{}",
            self.indent + 2,
            RenderType {
                ty: &self.pair.1,
                indent: self.indent + 2
            },
        )?;
        writeln!(f, "{}}},", self.indent + 1)?;
        write!(f, "{})", self.indent)
    }
}
