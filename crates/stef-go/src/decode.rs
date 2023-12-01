#![allow(clippy::too_many_lines)]

use std::fmt::{self, Display};

use stef_parser::{DataType, Fields, Generics, Struct, Type, Variant};

use crate::definition::{self, RenderGenericNames};

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
    pub(super) generics: &'a Generics<'a>,
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
        match self.0 {
            Fields::Named(named) => {
                for field in named {
                    writeln!(f, "\tfound{} := false", heck::AsUpperCamelCase(&field.name))?;
                }
            }
            Fields::Unnamed(unnamed) => {
                for (idx, _) in unnamed.iter().enumerate() {
                    writeln!(f, "\tfoundF{idx} := false")?;
                }
            }
            Fields::Unit => {}
        }

        Ok(())
    }
}

struct RenderFoundChecks<'a>(&'a Fields<'a>);

impl Display for RenderFoundChecks<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Fields::Named(named) => {
                for field in named {
                    writeln!(f, "\tif !found{} {{", heck::AsUpperCamelCase(&field.name))?;
                    writeln!(f, "\t\treturn nil, buf.MissingFieldError{{")?;
                    writeln!(f, "\t\t\tID:    {}", field.id.get())?;
                    writeln!(f, "\t\t\tField: \"{}\"", &field.name)?;
                    writeln!(f, "\t\t}}")?;
                    writeln!(f, "\t}}")?;
                }
            }
            Fields::Unnamed(unnamed) => {
                for (idx, field) in unnamed.iter().enumerate() {
                    writeln!(f, "\tif !foundF{idx} {{")?;
                    writeln!(f, "\t\treturn nil, buf.MissingFieldError{{")?;
                    writeln!(f, "\t\t\tID:    {}", field.id.get())?;
                    writeln!(f, "\t\t\tField: \"\"")?;
                    writeln!(f, "\t\t}}")?;
                    writeln!(f, "\t}}")?;
                }
            }
            Fields::Unit => {}
        }

        Ok(())
    }
}

struct RenderFields<'a>(&'a Fields<'a>);

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Fields::Named(named) => {
                for field in named {
                    writeln!(f, "\t\t\tcase {}:", field.id.get())?;
                    writeln!(
                        f,
                        "\t\t\t\tr2, value, err := {}",
                        RenderType {
                            ty: &field.ty,
                            indent: 4
                        }
                    )?;
                    writeln!(f, "\t\t\t\tif err != nil {{")?;
                    writeln!(f, "\t\t\t\t\treturn nil, err")?;
                    writeln!(f, "\t\t\t\t}}")?;
                    writeln!(f, "\t\t\t\tr = r2")?;
                    writeln!(
                        f,
                        "\t\t\t\tv.{} = value",
                        heck::AsUpperCamelCase(&field.name)
                    )?;
                    writeln!(
                        f,
                        "\t\t\t\tfound{} = true",
                        heck::AsUpperCamelCase(&field.name)
                    )?;
                }
            }
            Fields::Unnamed(unnamed) => {
                for (idx, field) in unnamed.iter().enumerate() {
                    writeln!(f, "\t\t\tcase {}:", field.id.get())?;
                    writeln!(
                        f,
                        "\t\t\t\tr2, value, err := {}",
                        RenderType {
                            ty: &field.ty,
                            indent: 4
                        }
                    )?;
                    writeln!(f, "\t\t\t\tif err != nil {{")?;
                    writeln!(f, "\t\t\t\t\treturn nil, err")?;
                    writeln!(f, "\t\t\t\t}}")?;
                    writeln!(f, "\t\t\t\tr = r2")?;
                    writeln!(f, "\t\t\t\tv.F{idx} = value")?;
                    writeln!(f, "\t\t\t\tfoundF{idx} = true")?;
                }
            }
            Fields::Unit => {}
        }

        Ok(())
    }
}

struct RenderType<'a> {
    ty: &'a Type<'a>,
    indent: usize,
}

impl Display for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.ty.value {
            DataType::Bool => write!(f, "buf.DecodeBool(r)"),
            DataType::U8 => write!(f, "buf.DecodeU8(r)"),
            DataType::U16 => write!(f, "buf.DecodeU16(r)"),
            DataType::U32 => write!(f, "buf.DecodeU32(r)"),
            DataType::U64 => write!(f, "buf.DecodeU64(r)"),
            DataType::U128 => write!(f, "buf.DecodeU128(r)"),
            DataType::I8 => write!(f, "buf.DecodeI8(r)"),
            DataType::I16 => write!(f, "buf.DecodeI16(r)"),
            DataType::I32 => write!(f, "buf.DecodeI32(r)"),
            DataType::I64 => write!(f, "buf.DecodeI64(r)"),
            DataType::I128 => write!(f, "buf.DecodeI128(r)"),
            DataType::F32 => write!(f, "buf.DecodeF32(r)"),
            DataType::F64 => write!(f, "buf.DecodeF64(r)"),
            DataType::String | DataType::StringRef | DataType::BoxString => {
                write!(f, "buf.DecodeString(r)")
            }
            DataType::Bytes | DataType::BytesRef | DataType::BoxBytes => {
                write!(f, "buf.DecodeBytes(r)")
            }
            DataType::Vec(ty) => {
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
            DataType::HashMap(kv) => {
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
            DataType::HashSet(ty) => {
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
            DataType::Option(ty) => {
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
            DataType::NonZero(ty) => match &ty.value {
                DataType::U8 => write!(f, "buf.DecodeNonZeroU8(r)"),
                DataType::U16 => write!(f, "buf.DecodeNonZeroU16(r)"),
                DataType::U32 => write!(f, "buf.DecodeNonZeroU32(r)"),
                DataType::U64 => write!(f, "buf.DecodeNonZeroU64(r)"),
                DataType::U128 => write!(f, "buf.DecodeNonZeroU128(r)"),
                DataType::I8 => write!(f, "buf.DecodeNonZeroI8(r)"),
                DataType::I16 => write!(f, "buf.DecodeNonZeroI16(r)"),
                DataType::I32 => write!(f, "buf.DecodeNonZeroI32(r)"),
                DataType::I64 => write!(f, "buf.DecodeNonZeroI64(r)"),
                DataType::I128 => write!(f, "buf.DecodeNonZeroI128(r)"),
                DataType::String | DataType::StringRef => write!(f, "buf.DecodeNonZeroString(r)"),
                DataType::Bytes | DataType::BytesRef => write!(f, "buf.DecodeNonZeroBytes(r)"),
                DataType::Vec(ty) => {
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
                DataType::HashMap(kv) => {
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
                DataType::HashSet(ty) => {
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
            DataType::Tuple(types) => match types.len() {
                n @ 2..=12 => {
                    writeln!(f, "func (r []byte) ([]byte, buf.Tuple{n}, error) {{")?;
                    for (idx, ty) in types.iter().enumerate() {
                        writeln!(
                            f,
                            "{:\t<indent$}r2, value{idx}, err := {}",
                            "",
                            RenderType {
                                ty,
                                indent: self.indent + 1
                            },
                            indent = self.indent + 1,
                        )?;
                        writeln!(
                            f,
                            "{:\t<indent$}if err != nil {{",
                            "",
                            indent = self.indent + 1,
                        )?;
                        writeln!(
                            f,
                            "{:\t<indent$}return nil, value, err",
                            "",
                            indent = self.indent + 2,
                        )?;
                        writeln!(f, "{:\t<indent$}}}", "", indent = self.indent + 1)?;
                        writeln!(f, "{:\t<indent$}r = r2", "", indent = self.indent + 1)?;
                        writeln!(
                            f,
                            "{:\t<indent$}tuple.F{idx} = value{idx}",
                            "",
                            indent = self.indent + 1,
                        )?;
                    }
                    writeln!(
                        f,
                        "{:\t<indent$}return r, tuple, nil",
                        "",
                        indent = self.indent + 1,
                    )?;
                    write!(f, "{:\t<indent$}}}(r)", "", indent = self.indent)
                }
                n => todo!("compiler should catch invalid tuple with {n} elements"),
            },
            DataType::Array(ty, size) => match *size {
                1..=32 => {
                    writeln!(
                        f,
                        "buf.DecodeArray{size}[{}](r, func(r []byte) ([]byte, {0}, error) {{",
                        definition::RenderType(ty),
                    )?;
                    writeln!(
                        f,
                        "{:\t<indent$}return {}",
                        "",
                        RenderType {
                            ty,
                            indent: self.indent + 1
                        },
                        indent = self.indent + 1,
                    )?;
                    write!(f, "{:\t<indent$}}})", "", indent = self.indent)
                }
                n => todo!("arrays with larger ({n}) sizes"),
            },
            DataType::External(_) => {
                writeln!(
                    f,
                    "func(r []byte) ([]byte, {}, error) {{",
                    definition::RenderType(self.ty)
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}var value {}",
                    "",
                    definition::RenderType(self.ty),
                    indent = self.indent + 1,
                )?;
                writeln!(
                    f,
                    "{:\t<indent$}return value.Decode(r)",
                    "",
                    indent = self.indent + 1,
                )?;
                writeln!(f, "{:\t<indent$}}}(r)", "", indent = self.indent)
            }
        }
    }
}

struct DecodeGenericSingle<'a> {
    name: &'static str,
    ty: &'a Type<'a>,
    indent: usize,
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
            "{:\t<indent$}return {}",
            "",
            RenderType {
                ty: self.ty,
                indent: self.indent + 1
            },
            indent = self.indent + 1,
        )?;
        write!(f, "{:\t<indent$}}})", "", indent = self.indent)
    }
}

struct DecodeGenericPair<'a> {
    name: &'static str,
    pair: &'a (Type<'a>, Type<'a>),
    indent: usize,
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
        writeln!(f, "{:\t<indent$}r,", "", indent = self.indent + 1)?;

        writeln!(
            f,
            "{:\t<indent$}func(r []byte) ([]byte, {}, error) {{",
            "",
            definition::RenderType(&self.pair.0),
            indent = self.indent + 1
        )?;
        writeln!(
            f,
            "{:\t<indent$}return {}",
            "",
            RenderType {
                ty: &self.pair.0,
                indent: self.indent + 2
            },
            indent = self.indent + 2,
        )?;
        writeln!(f, "{:\t<indent$}}},", "", indent = self.indent + 1)?;

        writeln!(
            f,
            "{:\t<indent$}func(r []byte) ([]byte, {}, error) {{",
            "",
            definition::RenderType(&self.pair.1),
            indent = self.indent + 1
        )?;
        writeln!(
            f,
            "{:\t<indent$}return {}",
            "",
            RenderType {
                ty: &self.pair.1,
                indent: self.indent + 2
            },
            indent = self.indent + 2,
        )?;
        writeln!(f, "{:\t<indent$}}},", "", indent = self.indent + 1)?;
        write!(f, "{:\t<indent$})", "", indent = self.indent)
    }
}
