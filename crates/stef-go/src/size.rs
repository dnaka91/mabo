#![allow(clippy::too_many_lines)]

use std::fmt::{self, write, Display, Write};

use stef_parser::{DataType, Fields, Generics, Struct, Type, Variant};

use crate::definition::{self, RenderGenericNames};

pub(super) struct RenderStruct<'a>(pub(super) &'a Struct<'a>);

struct GoFunc<'a, T>(&'a mut T);

impl<T: Write> GoFunc<'_, T> {
    fn receiver(self, name: impl Display, ty: impl Display) -> Result<Self, fmt::Error> {
        write!(self.0, " ({name} {ty})")?;
        Ok(self)
    }

    fn named(self, name: impl Display) -> Result<Self, fmt::Error> {
        write!(self.0, " {name}")?;
        Ok(self)
    }

    fn params(self, params: impl IntoIterator<Item = impl Display>) -> Result<Self, fmt::Error> {
        self.0.write_char('(')?;
        for (i, param) in params.into_iter().enumerate() {
            if i > 0 {
                self.0.write_str(", ")?;
            }
            write!(self.0, "{param}")?;
        }
        self.0.write_char(')')?;

        Ok(self)
    }

    fn no_params(self) -> Result<Self, fmt::Error> {
        self.0.write_str("()")?;
        Ok(self)
    }

    fn returns(
        self,
        returns: impl IntoIterator<
            Item = impl Display,
            IntoIter = impl ExactSizeIterator<Item = impl Display>,
        >,
    ) -> Result<Self, fmt::Error> {
        self.0.write_char(' ')?;

        let returns = returns.into_iter();
        let multi = returns.len() > 1;

        if multi {
            self.0.write_char('(')?;
        }
        for (i, ret) in returns.enumerate() {
            if i > 0 {
                self.0.write_str(", ")?;
            }
            write!(self.0, "{ret}")?;
        }
        if multi {
            self.0.write_char(')')?;
        }

        Ok(self)
    }

    fn body(self, build: impl FnOnce(usize, &mut T) -> fmt::Result) -> fmt::Result {
        self.0.write_str(" {\n")?;
        build(1, self.0)?;
        self.0.write_str("}")
    }
}

fn func<T: Write>(w: &mut T) -> Result<GoFunc<'_, T>, fmt::Error> {
    w.write_str("func")?;
    Ok(GoFunc(w))
}

struct GoCall<'a, T> {
    w: &'a mut T,
    multiline: bool,
    params_open: bool,
    params_count: usize,
}

impl<T: Write> GoCall<'_, T> {
    fn generics(self, types: impl Display) -> Result<Self, fmt::Error> {
        write!(self.w, "[{types}]")?;
        Ok(self)
    }

    fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    fn param(mut self, param: impl Display) -> Result<Self, fmt::Error> {
        if !self.params_open {
            self.w.write_char('(')?;
            self.params_open = true;
        }

        if self.params_count > 0 {
            self.w.write_str(", ")?;
        }

        write!(self.w, "{param}")?;
        self.params_count += 1;

        Ok(self)
    }

    fn param_with(mut self, build: impl FnOnce(&mut T) -> fmt::Result) -> Result<Self, fmt::Error> {
        if !self.params_open {
            self.w.write_char('(')?;
            self.params_open = true;
        }

        if self.params_count > 0 {
            self.w.write_str(", ")?;
        }

        build(self.w)?;
        self.params_count += 1;

        Ok(self)
    }

    fn end(self) -> fmt::Result {
        // if self.multiline {
        //     write_indent(self.w, self.indent)?;
        // }
        self.w.write_str(")\n")
    }
}

fn call<T: Write>(
    w: &mut T,
    indent: usize,
    name: impl Display,
) -> Result<GoCall<'_, T>, fmt::Error> {
    write_indent(w, indent)?;
    write!(w, "{name}")?;
    Ok(GoCall {
        w,
        multiline: false,
        params_open: false,
        params_count: 0,
    })
}

fn ensure_impl(w: &mut impl Write, interface: impl Display, ty: impl Display) -> fmt::Result {
    writeln!(w, "var _ {interface} = (*{ty})(nil)\n",)
}

fn write_indent(w: &mut impl Write, indent: usize) -> fmt::Result {
    write!(w, "{:\t<indent$}", "",)
}

fn assign(
    w: &mut impl Write,
    indent: usize,
    variable: impl Display,
    op: Op,
    expr: impl Display,
) -> fmt::Result {
    write_indent(w, indent)?;
    writeln!(w, "{variable} {op} {expr}")
}

fn assign_with<T: Write>(
    w: &mut T,
    indent: usize,
    variable: impl Display,
    op: Op,
    expr: impl FnOnce(usize, &mut T) -> fmt::Result,
) -> fmt::Result {
    write_indent(w, indent)?;
    write!(w, "{variable} {op} ")?;
    expr(indent, w)
}

enum Op {
    Init,
    Assign,
    AddAssign,
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Init => ":=",
            Self::Assign => "=",
            Self::AddAssign => "+=",
        })
    }
}

fn retval(
    w: &mut impl Write,
    indent: usize,
    values: impl IntoIterator<Item = impl Display>,
) -> fmt::Result {
    write_indent(w, indent)?;
    w.write_str("return ")?;
    for (i, value) in values.into_iter().enumerate() {
        if i > 0 {
            w.write_str(", ")?;
        }
        write!(w, "{value}")?;
    }
    w.write_char('\n')
}

impl Display for RenderStruct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ensure_impl(
            f,
            "buf.Size",
            format_args!(
                "{}{}",
                heck::AsUpperCamelCase(&self.0.name),
                RenderGenericNames {
                    generics: &self.0.generics,
                    fields_filter: None,
                }
            ),
        )?;

        func(f)?
            .receiver(
                "v",
                format_args!(
                    "*{}{}",
                    heck::AsUpperCamelCase(&self.0.name),
                    RenderGenericNames {
                        generics: &self.0.generics,
                        fields_filter: None,
                    }
                ),
            )?
            .named("Size")?
            .no_params()?
            .returns(["int"])?
            .body(|indent, f| {
                assign(f, indent, "size", Op::Init, 0)?;
                write!(f, "{}", RenderFields(&self.0.fields))?;
                retval(f, indent, ["size"])
            })
    }
}

pub(super) struct RenderEnumVariant<'a> {
    pub(super) enum_name: &'a str,
    pub(super) generics: &'a Generics<'a>,
    pub(super) variant: &'a Variant<'a>,
}

impl Display for RenderEnumVariant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ensure_impl(
            f,
            "buf.Size",
            format_args!(
                "{}_{}{}",
                heck::AsUpperCamelCase(self.enum_name),
                heck::AsUpperCamelCase(&self.variant.name),
                RenderGenericNames {
                    generics: self.generics,
                    fields_filter: Some(&self.variant.fields),
                }
            ),
        )?;

        func(f)?
            .receiver(
                "v",
                format_args!(
                    "*{}_{}{}",
                    heck::AsUpperCamelCase(self.enum_name),
                    heck::AsUpperCamelCase(&self.variant.name),
                    RenderGenericNames {
                        generics: self.generics,
                        fields_filter: Some(&self.variant.fields),
                    }
                ),
            )?
            .named("Size")?
            .no_params()?
            .returns(["int"])?
            .body(|indent, f| {
                assign(f, indent, "size", Op::Init, 0)?;
                write!(f, "{}", RenderFields(&self.variant.fields))?;
                retval(f, indent, ["size"])
            })
    }
}

struct RenderFields<'a>(&'a Fields<'a>);

impl Display for RenderFields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Fields::Named(named) => {
                for field in named {
                    if let DataType::Option(ty) = &field.ty.value {
                        assign_with(f, 1, "size", Op::AddAssign, |indent, f| {
                            call(f, 0, "buf.SizeFieldOption")?
                                .generics(definition::RenderType(ty))?
                                .param(field.id.get())?
                                .param(format_args!("&v.{}", heck::AsUpperCamelCase(&field.name)))?
                                .param_with(|f| {
                                    func(f)?
                                        .params([format_args!("v {}", definition::RenderType(ty))])?
                                        .returns(["[]byte"])?
                                        .body(|_indent, f| {
                                            retval(
                                                f,
                                                indent + 1,
                                                [RenderType {
                                                    ty,
                                                    name: "v",
                                                    indent: 2,
                                                }],
                                            )
                                        })
                                })?
                                .end()
                        })?;
                    } else {
                        assign_with(f, 1, "size", Op::AddAssign, |_, f| {
                            call(f, 0, "buf.SizeField")?
                                .param(field.id.get())?
                                .param_with(|f| {
                                    func(f)?.no_params()?.returns(["int"])?.body(|indent, f| {
                                        retval(
                                            f,
                                            indent + 1,
                                            [RenderType {
                                                ty: &field.ty,
                                                name: format_args!(
                                                    "v.{}",
                                                    heck::AsUpperCamelCase(&field.name)
                                                ),
                                                indent: 2,
                                            }],
                                        )
                                    })
                                })?
                                .end()
                        })?;
                    }
                }

                writeln!(f, "\tsize += buf.SizeU32(buf.EndMarker)")?;
            }
            Fields::Unnamed(unnamed) => {
                for (idx, field) in unnamed.iter().enumerate() {
                    if let DataType::Option(ty) = &field.ty.value {
                        writeln!(
                            f,
                            "\tsize += buf.SizeFieldOption[{}]({}, &v.F{idx}, func (v {0}) int \
                             {{\n\t\treturn {}\n\t}})",
                            definition::RenderType(ty),
                            field.id.get(),
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
                            field.id.get(),
                            RenderType {
                                ty: &field.ty,
                                name: format_args!("v.F{idx}"),
                                indent: 2,
                            },
                        )?;
                    }
                }

                assign_with(f, 1, "size", Op::AddAssign, |_, f| {
                    call(f, 0, "buf.SizeU32")?.param("buf.EndMarker")?.end()
                })?;
            }
            Fields::Unit => {}
        }

        Ok(())
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
        match &self.ty.value {
            DataType::Bool => call(f, 0, "buf.SizeBool")?.param(&self.name)?.end(),
            DataType::U8 => write!(f, "buf.SizeU8({})", self.name),
            DataType::U16 => write!(f, "buf.SizeU16({})", self.name),
            DataType::U32 => write!(f, "buf.SizeU32({})", self.name),
            DataType::U64 => write!(f, "buf.SizeU64({})", self.name),
            DataType::U128 => write!(f, "buf.SizeU128({})", self.name),
            DataType::I8 => write!(f, "buf.SizeI8({})", self.name),
            DataType::I16 => write!(f, "buf.SizeI16({})", self.name),
            DataType::I32 => write!(f, "buf.SizeI32({})", self.name),
            DataType::I64 => write!(f, "buf.SizeI64({})", self.name),
            DataType::I128 => write!(f, "buf.SizeI128({})", self.name),
            DataType::F32 => write!(f, "buf.SizeF32({})", self.name),
            DataType::F64 => write!(f, "buf.SizeF64({})", self.name),
            DataType::String | DataType::StringRef | DataType::BoxString => {
                write!(f, "buf.SizeString({})", self.name)
            }
            DataType::Bytes | DataType::BytesRef | DataType::BoxBytes => {
                write!(f, "buf.SizeBytes({})", self.name)
            }
            DataType::Vec(ty) => call(f, 0, "buf.SizeVec")?
                .generics(definition::RenderType(ty))?
                .param(&self.name)?
                .param_with(|f| {
                    func(f)?
                        .params([format_args!("v {}", definition::RenderType(ty))])?
                        .returns(["int"])?
                        .body(|indent, f| {
                            retval(
                                f,
                                indent+1,
                                [RenderType {
                                    ty,
                                    name: "v",
                                    indent: indent + 1,
                                }],
                            )
                        })
                })?
                .end(),
            DataType::HashMap(kv) => {
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
            DataType::HashSet(ty) => {
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
            DataType::Option(ty) => {
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
            DataType::NonZero(ty) => match &ty.value {
                DataType::U8 => write!(f, "buf.SizeU8({}.Get())", self.name),
                DataType::U16 => write!(f, "buf.SizeU16({}.Get())", self.name),
                DataType::U32 => write!(f, "buf.SizeU32({}.Get())", self.name),
                DataType::U64 => write!(f, "buf.SizeU64({}.Get())", self.name),
                DataType::U128 => write!(f, "buf.SizeU128({}.Get())", self.name),
                DataType::I8 => write!(f, "buf.SizeI8({}.Get())", self.name),
                DataType::I16 => write!(f, "buf.SizeI16({}.Get())", self.name),
                DataType::I32 => write!(f, "buf.SizeI32({}.Get())", self.name),
                DataType::I64 => write!(f, "buf.SizeI64({}.Get())", self.name),
                DataType::I128 => write!(f, "buf.SizeI128({}.Get())", self.name),
                DataType::String
                | DataType::StringRef
                | DataType::Bytes
                | DataType::BytesRef
                | DataType::Vec(_)
                | DataType::HashMap(_)
                | DataType::HashSet(_) => write!(
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
            DataType::Tuple(types) => match types.len() {
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
            DataType::Array(ty, size) => match *size {
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
            DataType::External(_) => write!(f, "{}.Size()", self.name),
        }
    }
}
