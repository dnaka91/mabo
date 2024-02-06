#![expect(clippy::too_many_lines)]

use std::fmt::{self, Display};

use mabo_compiler::simplify::{FieldKind, Fields, Struct, Type, Variant};

use crate::{
    Indent,
    definition::{self, RenderGenericNames},
};

pub(super) struct RenderStruct<'a>(pub(super) &'a Struct<'a>);

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "var _ buf.Decode = (*{}{})(nil)\n",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;

        writeln!(
            f,
            "func (v *{}{}) Decode(r []byte) ([]byte, error) {{",
            heck::AsUpperCamelCase(&self.0.name),
            RenderGenericNames {
                generics: &self.0.generics,
                fields_filter: None,
            }
        )?;
        writeln!(f, "{}", RenderFieldVars(&self.0.fields))?;
        writeln!(f, "\tfor len(r) > 0 {{")?;
        writeln!(f, "\t\tr2, id, err := buf.DecodeID(r)")?;
        writeln!(f, "\t\tif err != nil {{")?;
        writeln!(f, "\t\t\treturn nil, err")?;
        writeln!(f, "\t\t}}")?;
        writeln!(f, "\t\tr = r2\n")?;
        writeln!(f, "\t\tswitch id {{")?;
        write!(f, "{}", RenderFields(&self.0.fields))?;
        writeln!(f, "\t\t\tcase buf.EndMarker:")?;
        writeln!(f, "\t\t\t\tbreak")?;
        writeln!(f, "\t\t}}")?;
        writeln!(f, "\t}}\n")?;
        write!(f, "{}", RenderFoundChecks(&self.0.fields))?;
        writeln!(f, "\n\treturn r, nil\n}}")
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
            "var _ buf.Decode = (*{}_{}{})(nil)\n",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;

        writeln!(
            f,
            "func (v *{}_{}{}) Decode(r []byte) ([]byte, error) {{",
            heck::AsUpperCamelCase(self.enum_name),
            heck::AsUpperCamelCase(&self.variant.name),
            RenderGenericNames {
                generics: self.generics,
                fields_filter: Some(&self.variant.fields),
            }
        )?;
        writeln!(f, "{}", RenderFieldVars(&self.variant.fields))?;
        writeln!(f, "\tfor len(r) > 0 {{")?;
        writeln!(f, "\t\tr2, id, err := buf.DecodeID(r)")?;
        writeln!(f, "\t\tif err != nil {{")?;
        writeln!(f, "\t\t\treturn nil, err")?;
        writeln!(f, "\t\t}}")?;
        writeln!(f, "\t\tr = r2\n")?;
        writeln!(f, "\t\tswitch id {{")?;
        write!(f, "{}", RenderFields(&self.variant.fields))?;
        writeln!(f, "\t\t\tcase buf.EndMarker:")?;
        writeln!(f, "\t\t\t\tbreak")?;
        writeln!(f, "\t\t}}")?;
        writeln!(f, "\t}}\n")?;
        write!(f, "{}", RenderFoundChecks(&self.variant.fields))?;
        writeln!(f, "\n\treturn r, nil\n}}")
    }
}

struct RenderFieldVars<'a>(&'a Fields<'a>);

impl Display for RenderFieldVars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in &*self.0.fields {
            writeln!(f, "\tfound{} := false", heck::AsUpperCamelCase(&field.name))?;
        }

        Ok(())
    }
}

struct RenderFoundChecks<'a>(&'a Fields<'a>);

impl Display for RenderFoundChecks<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in &*self.0.fields {
            writeln!(f, "\tif !found{} {{", heck::AsUpperCamelCase(&field.name))?;
            writeln!(f, "\t\treturn nil, buf.MissingFieldError{{")?;
            writeln!(f, "\t\t\tID:    {},", field.id)?;
            writeln!(
                f,
                "\t\t\tField: \"{}\",",
                if self.0.kind == FieldKind::Named {
                    &field.name
                } else {
                    ""
                }
            )?;
            writeln!(f, "\t\t}}")?;
            writeln!(f, "\t}}")?;
        }

        Ok(())
    }
}

struct RenderFields<'a>(&'a Fields<'a>);

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in &*self.0.fields {
            writeln!(f, "\t\t\tcase {}:", field.id)?;
            writeln!(
                f,
                "\t\t\t\tr2, value, err := {}",
                RenderType {
                    ty: &field.ty,
                    indent: Indent(4),
                },
            )?;
            writeln!(f, "\t\t\t\tif err != nil {{")?;
            writeln!(f, "\t\t\t\t\treturn nil, err")?;
            writeln!(f, "\t\t\t\t}}")?;
            writeln!(f, "\t\t\t\tr = r2")?;
            writeln!(
                f,
                "\t\t\t\tv.{} = value",
                heck::AsUpperCamelCase(&field.name),
            )?;
            writeln!(
                f,
                "\t\t\t\tfound{} = true",
                heck::AsUpperCamelCase(&field.name),
            )?;
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
            Type::Bool => write!(f, "buf.DecodeBool(r)"),
            Type::U8 => write!(f, "buf.DecodeU8(r)"),
            Type::U16 => write!(f, "buf.DecodeU16(r)"),
            Type::U32 => write!(f, "buf.DecodeU32(r)"),
            Type::U64 => write!(f, "buf.DecodeU64(r)"),
            Type::U128 => write!(f, "buf.DecodeU128(r)"),
            Type::UBig => write!(f, "buf.DecodeUBig(r)"),
            Type::I8 => write!(f, "buf.DecodeI8(r)"),
            Type::I16 => write!(f, "buf.DecodeI16(r)"),
            Type::I32 => write!(f, "buf.DecodeI32(r)"),
            Type::I64 => write!(f, "buf.DecodeI64(r)"),
            Type::I128 => write!(f, "buf.DecodeI128(r)"),
            Type::IBig => write!(f, "buf.DecodeIBig(r)"),
            Type::F32 => write!(f, "buf.DecodeF32(r)"),
            Type::F64 => write!(f, "buf.DecodeF64(r)"),
            Type::String | Type::StringRef | Type::BoxString => {
                write!(f, "buf.DecodeString(r)")
            }
            Type::Bytes | Type::BytesRef | Type::BoxBytes => {
                write!(f, "buf.DecodeBytes(r)")
            }
            Type::Vec(ty) => {
                write!(
                    f,
                    "{}",
                    DecodeGenericSingle {
                        name: "DecodeVec",
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
                        name: "DecodeHashMap",
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
                        name: "DecodeHashSet",
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
                        name: "DecodeOption",
                        ty,
                        indent: self.indent,
                    }
                )
            }
            Type::NonZero(ty) => match &**ty {
                Type::U8 => write!(f, "buf.DecodeNonZeroU8(r)"),
                Type::U16 => write!(f, "buf.DecodeNonZeroU16(r)"),
                Type::U32 => write!(f, "buf.DecodeNonZeroU32(r)"),
                Type::U64 => write!(f, "buf.DecodeNonZeroU64(r)"),
                Type::U128 => write!(f, "buf.DecodeNonZeroU128(r)"),
                Type::UBig => write!(f, "buf.DecodeNonZeroUBig(r)"),
                Type::I8 => write!(f, "buf.DecodeNonZeroI8(r)"),
                Type::I16 => write!(f, "buf.DecodeNonZeroI16(r)"),
                Type::I32 => write!(f, "buf.DecodeNonZeroI32(r)"),
                Type::I64 => write!(f, "buf.DecodeNonZeroI64(r)"),
                Type::I128 => write!(f, "buf.DecodeNonZeroI128(r)"),
                Type::IBig => write!(f, "buf.DecodeNonZeroIBig(r)"),
                Type::String | Type::StringRef => write!(f, "buf.DecodeNonZeroString(r)"),
                Type::Bytes | Type::BytesRef => write!(f, "buf.DecodeNonZeroBytes(r)"),
                Type::Vec(ty) => {
                    write!(
                        f,
                        "{}",
                        DecodeGenericSingle {
                            name: "DecodeNonZeroVec",
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
                            name: "DecodeNonZeroHashMap",
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
                            name: "DecodeNonZeroHashSet",
                            ty,
                            indent: self.indent,
                        }
                    )
                }
                ty => todo!("compiler should catch invalid {ty:?} type"),
            },
            Type::Tuple(types) => match types.len() {
                n @ 2..=12 => {
                    writeln!(f, "func (r []byte) ([]byte, buf.Tuple{n}, error) {{")?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{}r2, value{idx}, err := {}",
                            self.indent + 1,
                            RenderType {
                                ty,
                                indent: self.indent + 1,
                            },
                        )?;
                        writeln!(f, "{}if err != nil {{", self.indent + 1)?;
                        writeln!(f, "{}return nil, value, err", self.indent + 2)?;
                        writeln!(f, "{}}}", self.indent + 1)?;
                        writeln!(f, "{}r = r2", self.indent + 1)?;
                        writeln!(f, "{}tuple.F{idx} = value{idx}", self.indent + 1)?;
                    }
                    writeln!(f, "{}return r, tuple, nil", self.indent + 1)?;
                    write!(f, "{}}}(r)", self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            Type::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(
                        f,
                        "buf.DecodeArray{size}[{}](r, func(r []byte) ([]byte, {0}, error) {{",
                        definition::RenderType(ty),
                    )?;
                    writeln!(
                        f,
                        "{}return {}",
                        self.indent + 1,
                        RenderType {
                            ty,
                            indent: self.indent + 1,
                        },
                    )?;
                    write!(f, "{}}})", self.indent)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            Type::External(_) => {
                writeln!(
                    f,
                    "func(r []byte) ([]byte, {}, error) {{",
                    definition::RenderType(self.ty)
                )?;
                writeln!(
                    f,
                    "{}var value {}",
                    self.indent + 1,
                    definition::RenderType(self.ty),
                )?;
                writeln!(f, "{}return value.Decode(r)", self.indent + 1)?;
                writeln!(f, "{}}}(r)", self.indent)
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
        writeln!(
            f,
            "buf.{}[{}](r, func(r []byte) ([]byte, {1}, error) {{",
            self.name,
            definition::RenderType(self.ty),
        )?;
        writeln!(
            f,
            "{}return {}",
            self.indent + 1,
            RenderType {
                ty: self.ty,
                indent: self.indent + 1,
            },
        )?;
        write!(f, "{}}})", self.indent)
    }
}

struct DecodeGenericPair<'a> {
    name: &'static str,
    pair: &'a (Type<'a>, Type<'a>),
    indent: Indent,
}

impl Display for DecodeGenericPair<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "buf.{}[{}, {}](",
            self.name,
            definition::RenderType(&self.pair.0),
            definition::RenderType(&self.pair.1)
        )?;
        writeln!(f, "{}r,", self.indent + 1)?;

        writeln!(
            f,
            "{}func(r []byte) ([]byte, {}, error) {{",
            self.indent + 1,
            definition::RenderType(&self.pair.0),
        )?;
        writeln!(
            f,
            "{}return {}",
            self.indent + 2,
            RenderType {
                ty: &self.pair.0,
                indent: self.indent + 2,
            },
        )?;
        writeln!(f, "{}}},", self.indent + 1)?;

        writeln!(
            f,
            "{}func(r []byte) ([]byte, {}, error) {{",
            self.indent + 1,
            definition::RenderType(&self.pair.1),
        )?;
        writeln!(
            f,
            "{}return {}",
            self.indent + 2,
            RenderType {
                ty: &self.pair.1,
                indent: self.indent + 2,
            },
        )?;
        writeln!(f, "{}}},", self.indent + 1)?;
        write!(f, "{})", self.indent)
    }
}
