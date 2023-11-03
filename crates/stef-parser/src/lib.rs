//! Parser for `STEF` schema files.
//!
//! The main entry point for schema parsing is the [`Schema::parse`] function. For convenience, the
//! [`from_str`] and [`from_slice`] functions are provided.
//!
//! # Example
//!
//! Parse a basic `STEF` schema and print it back out.
//!
//! ```
//! let schema = stef_parser::Schema::parse("struct Sample(u32 @1)", None).unwrap();
//!
//! // Pretty print the schema itself
//! println!("{schema}");
//! // Print the data structures themselves.
//! println!("{schema:#?}");
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]
#![warn(missing_docs, clippy::pedantic)]

use std::{
    fmt::{self, Display},
    ops::Range,
    path::{Path, PathBuf},
};

pub use miette::{Diagnostic, LabeledSpan};
use miette::{IntoDiagnostic, NamedSource, Result};
use winnow::Parser;

use self::error::ParseSchemaError;

pub mod error;
mod ext;
mod highlight;
mod location;
mod parser;

trait Print {
    /// Default indentation, 4 spaces.
    const INDENT: &'static str = "    ";

    /// Write to the given formatter (like [`Display::fmt`]) but in addition, take the current
    /// indentation level into account.
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result;

    /// Helper to write out the indentation for the given level.
    fn indent(f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        for _ in 0..level {
            f.write_str(Self::INDENT)?;
        }

        Ok(())
    }
}

/// Source code span that marks the location of any element in the schema that it was parsed from.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

/// Implemented by any parsed schema element that can report the source code span that it originated
/// from. This helps during error reporting to visualize the exact location of a problem.
pub trait Spanned {
    /// Get the source code span of this schema element.
    fn span(&self) -> Span;
}

/// Shorthand for calling [`Schema::parse`].
///
/// # Errors
///
/// Fails if the schema is not proper. The returned error will try to describe the problem as
/// precise as possible.
pub fn from_str<'a>(schema: &'a str, path: Option<&Path>) -> Result<Schema<'a>> {
    Schema::parse(schema, path).into_diagnostic()
}

/// Shorthand for calling [`Schema::parse`], but converts the byte slice to valid [`&str`] first.
///
/// # Errors
///
/// Fails if the schema is not proper. The returned error will try to describe the problem as
/// precise as possible. Or, in case the given bytes are not valid UTF-8.
pub fn from_slice<'a>(schema: &'a [u8], path: Option<&Path>) -> Result<Schema<'a>> {
    let s = std::str::from_utf8(schema).into_diagnostic()?;
    Schema::parse(s, path).into_diagnostic()
}

/// Uppermost element, describing a single _`STEF` Schema_ file.
#[derive(Debug, PartialEq)]
pub struct Schema<'a> {
    /// Physical location of the file that contains the schema source code.
    ///
    /// Might be missing if the schema originated from an inline string or otherwise omitted during
    /// the parsing process.
    pub path: Option<PathBuf>,
    /// Original source code form which this schema was parsed.
    pub source: &'a str,
    /// List of all the definitions that make up the schema.
    pub definitions: Vec<Definition<'a>>,
}

impl<'a> Schema<'a> {
    /// Try to parse the given schema.
    ///
    /// The optional path is not necessary, but can help to improve the error messages that are
    /// printed out on the terminal in case of an invalid schema. Effectively that means messages
    /// will include direct links to the files.
    ///
    /// # Errors
    ///
    /// Fails if the schema is not proper. The returned error will try to describe the problem as
    /// precise as possible.
    pub fn parse(input: &'a str, path: Option<&Path>) -> Result<Self, ParseSchemaError> {
        parser::parse_schema
            .parse(winnow::Located::new(input))
            .map(|mut schema| {
                schema.path = path.map(ToOwned::to_owned);
                schema
            })
            .map_err(|e| ParseSchemaError {
                source_code: NamedSource::new(
                    path.map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                    input.to_owned(),
                ),
                cause: e.into_inner(),
            })
    }
}

impl<'a> Display for Schema<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for definition in &self.definitions {
            writeln!(f, "{definition}")?;
        }
        Ok(())
    }
}

/// Possible elements that can appear inside a [`Schema`] or [`Module`].
#[derive(Debug, PartialEq)]
pub enum Definition<'a> {
    /// Module declaration to organize other definitions into scopes.
    Module(Module<'a>),
    /// Data structure.
    Struct(Struct<'a>),
    /// Enum definition.
    Enum(Enum<'a>),
    /// Type aliasing definition.
    TypeAlias(TypeAlias<'a>),
    /// Const value declaration.
    Const(Const<'a>),
    /// Import declaration of other schemas.
    Import(Import<'a>),
}

impl<'a> Print for Definition<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            Definition::Module(v) => v.print(f, level),
            Definition::Struct(v) => v.print(f, level),
            Definition::Enum(v) => v.print(f, level),
            Definition::TypeAlias(v) => v.print(f, level),
            Definition::Const(v) => v.print(f, level),
            Definition::Import(v) => v.print(f, level),
        }
    }
}

impl<'a> Display for Definition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

impl<'a> Definition<'a> {
    fn with_comment(mut self, comment: Comment<'a>) -> Self {
        match &mut self {
            Definition::Module(m) => m.comment = comment,
            Definition::Struct(s) => s.comment = comment,
            Definition::Enum(e) => e.comment = comment,
            Definition::TypeAlias(a) => a.comment = comment,
            Definition::Const(c) => c.comment = comment,
            Definition::Import(_) => {}
        }
        self
    }

    fn with_attributes(mut self, attributes: Attributes<'a>) -> Self {
        match &mut self {
            Definition::Struct(s) => s.attributes = attributes,
            Definition::Enum(e) => e.attributes = attributes,
            Definition::Module(_)
            | Definition::TypeAlias(_)
            | Definition::Const(_)
            | Definition::Import(_) => {}
        }
        self
    }
}

/// Scoping mechanism to categorize elements.
///
/// ```txt
/// mod my_mod {
///     struct Sample(u32 @1)
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct Module<'a> {
    /// Optional module-level comment.
    pub comment: Comment<'a>,
    /// Unique name of the module, within the current scope.
    pub name: Name<'a>,
    /// List of definitions that are scoped within this module.
    pub definitions: Vec<Definition<'a>>,
}

impl<'a> Print for Module<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            definitions,
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        writeln!(f, "mod {name} {{")?;

        for (i, definition) in definitions.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            definition.print(f, level + 1)?;
        }

        Self::indent(f, level)?;
        f.write_str("}\n")
    }
}

impl<'a> Display for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Rust-ish struct.
///
/// ```txt
/// /// Named
/// struct Sample {
///     value: u32 @1,
/// }
///
/// /// Unnamed
/// struct Sample(u32 @1)
///
/// /// Unit
/// struct Sample
/// ```
#[derive(Debug, PartialEq)]
pub struct Struct<'a> {
    /// Optional struct-level comment.
    pub comment: Comment<'a>,
    /// Optional attributes to customize the behavior.
    pub attributes: Attributes<'a>,
    /// Unique name for this struct (within its scope).
    pub name: Name<'a>,
    /// Potential generics.
    pub generics: Generics<'a>,
    /// Fields of the struct, if any.
    pub fields: Fields<'a>,
}

impl<'a> Print for Struct<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let indent = Self::INDENT.repeat(level);
        let Self {
            comment,
            attributes,
            name,
            generics,
            fields: kind,
        } = self;

        comment.print(f, level)?;
        attributes.print(f, level)?;
        write!(f, "{indent}struct {name}{generics}")?;
        kind.print(f, level)?;
        f.write_str("\n")
    }
}

impl<'a> Display for Struct<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Rust-ish enum.
///
/// ```txt
/// /// Optional comment
/// enum Sample {
///     /// Unit variant
///     One @1,
///     /// Unnamed (tuple) variant
///     Two(u8 @1) @2,
///     /// Named (struct) variant
///     Three {
///         value: u8 @1,
///     } @3,
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    /// Optional enum-level comment.
    pub comment: Comment<'a>,
    /// Optional attributes to customize the behavior.
    pub attributes: Attributes<'a>,
    /// Unique name for this enum, within its current scope.
    pub name: Name<'a>,
    /// Potential generics.
    pub generics: Generics<'a>,
    /// List of possible variants that the enum can represent.
    pub variants: Vec<Variant<'a>>,
}

impl<'a> Print for Enum<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            attributes,
            name,
            generics,
            variants,
        } = self;

        comment.print(f, level)?;
        attributes.print(f, level)?;

        Self::indent(f, level)?;
        writeln!(f, "enum {name}{generics} {{")?;

        for variant in variants {
            variant.print(f, level + 1)?;
            f.write_str("\n")?;
        }

        Self::indent(f, level)?;
        f.write_str("}\n")
    }
}

impl<'a> Display for Enum<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Single variant of an enum.
#[derive(Debug, Eq, PartialEq)]
pub struct Variant<'a> {
    /// Optional variant-level comment.
    pub comment: Comment<'a>,
    /// Unique for this variant, within the enum it belongs to.
    pub name: Name<'a>,
    /// Fields of this variant, if any.
    pub fields: Fields<'a>,
    /// Identifier for this variant, that must be unique within the current enum.
    pub id: Id,
    /// Source code location.
    span: Span,
}

impl<'a> Print for Variant<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            fields,
            id,
            span: _,
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        f.write_str(name.get())?;

        fields.print(f, level)?;
        write!(f, " {id},")
    }
}

impl<'a> Spanned for Variant<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Display for Variant<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Alias (re-name) from one type to another.
///
/// ```txt
/// /// Basic
/// type A = B;
///
/// /// With generics
/// type A<T> = hash_map<u32, T>;
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct TypeAlias<'a> {
    /// Optional comment.
    pub comment: Comment<'a>,
    /// Unique name of the type alias within the current scope.
    pub name: Name<'a>,
    /// Potential generic type arguments.
    pub generics: Generics<'a>,
    /// Original type that is being aliased.
    pub target: Type<'a>,
}

impl<'a> Print for TypeAlias<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            generics,
            target,
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        write!(f, "type {name}{generics} = {target};")
    }
}

/// Possible kinds in which a the fields of a struct or enum variant can be represented.
#[derive(Debug, Eq, PartialEq)]
pub enum Fields<'a> {
    /// List of named fields.
    ///
    /// ```txt
    /// Sample {
    ///     a: u8 @1,
    ///     b: bool @2,
    ///     c: i32 @3,
    /// }
    /// ```
    Named(Vec<NamedField<'a>>),
    /// List of types without an explicit name.
    ///
    /// ```txt
    /// Sample(u8 @1, bool @2, i32 @3)
    /// ```
    Unnamed(Vec<UnnamedField<'a>>),
    /// No attached value.
    ///
    /// ```txt
    /// Sample
    /// ```
    Unit,
}

impl<'a> Print for Fields<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            Fields::Named(fields) => {
                f.write_str(" {\n")?;

                for field in fields {
                    field.print(f, level + 1)?;
                    f.write_str(",\n")?;
                }

                Self::indent(f, level)?;
                f.write_str("}")
            }
            Fields::Unnamed(elements) => concat(f, "(", elements, ", ", ")"),
            Fields::Unit => Ok(()),
        }
    }
}

impl<'a> Display for Fields<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Single named field.
///
/// ```txt
/// field: u32 @1
/// ┬────  ┬── ┬─
/// │      │   ╰─── ID
/// │      ╰─────── Type
/// ╰────────────── Name
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct NamedField<'a> {
    /// Optional field-level comment.
    pub comment: Comment<'a>,
    /// Unique name for this field, within the current element.
    pub name: Name<'a>,
    /// Data type that defines the shape of the contained data.
    pub ty: Type<'a>,
    /// Identifier for this field, that must be unique within the current element.
    pub id: Id,
    /// Source code location.
    span: Span,
}

impl<'a> Print for NamedField<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            ty,
            id,
            span: _,
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        write!(f, "{name}: {ty} {id}")
    }
}

impl<'a> Spanned for NamedField<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Display for NamedField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Single unnamed field.
///
/// ```txt
/// u32 @1
/// ┬── ┬─
/// │   ╰─── ID
/// ╰─────── Type
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct UnnamedField<'a> {
    /// Data type that defines the shape of the contained data.
    pub ty: Type<'a>,
    /// Identifier for this field, that must be unique within the current element.
    pub id: Id,
    /// Source code location.
    span: Span,
}

impl<'a> Spanned for UnnamedField<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Display for UnnamedField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { ty, id, span: _ } = self;
        write!(f, "{ty} {id}")
    }
}

/// Comments above any other element.
///
/// ```txt
/// /// This is a comment.
///     ┬─────────────────
///     ╰─── Content
/// ```
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Comment<'a>(pub Vec<&'a str>);

impl<'a> Print for Comment<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let lines = &self.0;

        for line in lines {
            Self::indent(f, level)?;
            writeln!(f, "/// {line}")?;
        }

        Ok(())
    }
}

impl<'a> Display for Comment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Collection of attributes, aggregated together into a single declaration block.
#[derive(Debug, Default, PartialEq)]
pub struct Attributes<'a>(pub Vec<Attribute<'a>>);

impl<'a> Print for Attributes<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        let values = &self.0;

        Self::indent(f, level)?;
        concat(f, "#[", values, ", ", "]\n")
    }
}

impl<'a> Display for Attributes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Single attribute, that describes metadata for the attached element.
#[derive(Debug, PartialEq)]
pub struct Attribute<'a> {
    /// Identifier of the attribute.
    pub name: &'a str,
    /// Potential value(s) associated with the attribute.
    pub value: AttributeValue<'a>,
}

impl<'a> Print for Attribute<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let indent = Self::INDENT.repeat(level);
        let Self { name, value } = self;

        write!(f, "{indent}{name}{value}")
    }
}

impl<'a> Display for Attribute<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Value of an [`Attribute`] that can take one of several shapes.
#[derive(Debug, PartialEq)]
pub enum AttributeValue<'a> {
    /// No value, the attribute is representative by itself.
    Unit,
    /// Single literal value.
    Single(Literal),
    /// Multiple values, represented as sub-attributes.
    Multi(Vec<Attribute<'a>>),
}

impl<'a> Print for AttributeValue<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, _level: usize) -> fmt::Result {
        match self {
            Self::Unit => Ok(()),
            Self::Single(lit) => write!(f, " = {lit}"),
            Self::Multi(attrs) => concat(f, "(", attrs, ", ", ")"),
        }
    }
}

impl<'a> Display for AttributeValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// The data type which describes the shape of a field through its [`Self::value`] value, and
/// additionally carries the source span for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type<'a> {
    /// Possible data type of the field.
    pub value: DataType<'a>,
    /// Source code location.
    span: Span,
}

impl<'a> Spanned for Type<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Display for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'a> From<(DataType<'a>, Range<usize>)> for Type<'a> {
    fn from((value, span): (DataType<'a>, Range<usize>)) -> Self {
        Self {
            value,
            span: span.into(),
        }
    }
}

/// Possible data type that describes the shape of a field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataType<'a> {
    /// Boolean `true` or `false`.
    Bool,
    /// 8-bit unsigned integer.
    U8,
    /// 16-bit unsigned integer.
    U16,
    /// 32-bit unsigned integer.
    U32,
    /// 64-bit unsigned integer.
    U64,
    /// 128-bit unsigned integer.
    U128,
    /// 8-bit signed integer.
    I8,
    /// 16-bit signed integer.
    I16,
    /// 32-bit signed integer.
    I32,
    /// 64-bit signed integer.
    I64,
    /// 128-bit signed integer.
    I128,
    /// 32-bit floating point number.
    F32,
    /// 64-bit floating point number.
    F64,
    /// UTF-8 encoded string.
    String,
    /// Reference version of an UTF-8 encoded string.
    StringRef,
    /// Vector of `u8` bytes.
    Bytes,
    /// Reference version (slice) of `u8` bytes.
    BytesRef,
    /// Vector of another data type.
    Vec(Box<Type<'a>>),
    /// Key-value hash map of data types.
    HashMap(Box<(Type<'a>, Type<'a>)>),
    /// Hash set of data types (each entry is unique).
    HashSet(Box<Type<'a>>),
    /// Optional value.
    Option(Box<Type<'a>>),
    /// Non-zero value.
    /// - Integers: `n > 0`
    /// - Collections: `len() > 0`
    NonZero(Box<Type<'a>>),
    /// Boxed version of a string that is immutable.
    BoxString,
    /// Boxed version of a byte vector that is immutable.
    BoxBytes,
    /// Fixed size list of up to 12 types.
    Tuple(Vec<Type<'a>>),
    /// Continuous list of values with a single time and known length.
    Array(Box<Type<'a>>, u32),
    /// Any external, non-standard data type (like a user defined struct or enum).
    External(ExternalType<'a>),
}

impl<'a> Display for DataType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => f.write_str("bool"),
            Self::U8 => f.write_str("u8"),
            Self::U16 => f.write_str("u16"),
            Self::U32 => f.write_str("u32"),
            Self::U64 => f.write_str("u64"),
            Self::U128 => f.write_str("u128"),
            Self::I8 => f.write_str("i8"),
            Self::I16 => f.write_str("i16"),
            Self::I32 => f.write_str("i32"),
            Self::I64 => f.write_str("i64"),
            Self::I128 => f.write_str("i128"),
            Self::F32 => f.write_str("f32"),
            Self::F64 => f.write_str("f64"),
            Self::String => f.write_str("string"),
            Self::StringRef => f.write_str("&string"),
            Self::Bytes => f.write_str("bytes"),
            Self::BytesRef => f.write_str("&bytes"),
            Self::Vec(t) => write!(f, "vec<{t}>"),
            Self::HashMap(kv) => write!(f, "hash_map<{}, {}>", kv.0, kv.1),
            Self::HashSet(t) => write!(f, "hash_set<{t}>"),
            Self::Option(t) => write!(f, "option<{t}>"),
            Self::NonZero(t) => write!(f, "non_zero<{t}>"),
            Self::BoxString => f.write_str("box<string>"),
            Self::BoxBytes => f.write_str("box<bytes>"),
            Self::Tuple(l) => concat(f, "(", l, ", ", ")"),
            Self::Array(t, size) => write!(f, "[{t}; {size}]"),
            Self::External(t) => t.fmt(f),
        }
    }
}

/// Type that is not part of the built-in list of types.
///
/// This is usually a user-defined type like a struct or an enum. However, this can be the name of
/// a generic as well, as the type's origin is unknown at this point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalType<'a> {
    /// Optional path, if the type wasn't fully imported with a `use` statement.
    pub path: Vec<Name<'a>>,
    /// Unique name of the type within the current scope (or the module if prefixed with a path).
    pub name: Name<'a>,
    /// Potential generic type arguments.
    pub generics: Vec<Type<'a>>,
}

impl<'a> Display for ExternalType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            path,
            name,
            generics,
        } = self;

        for segment in path {
            write!(f, "{segment}::")?;
        }
        name.fmt(f)?;
        concat(f, "<", generics, ", ", ">")
    }
}

/// Container of generic arguments for an element.
///
/// ```txt
/// <A, B, ...>
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Generics<'a>(pub Vec<Name<'a>>);

impl<'a> Display for Generics<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        concat(f, "<", &self.0, ", ", ">")
    }
}

/// Unique identifier for an element.
///
/// ```txt
/// @1
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Id {
    /// Raw integer value.
    value: u32,
    /// Source code location.
    span: Span,
}

impl Id {
    /// Retrieve the raw integer value of this identifier.
    #[must_use]
    pub const fn get(&self) -> u32 {
        self.value
    }
}

impl Spanned for Id {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { value, span: _ } = *self;
        write!(f, "@{value}")
    }
}

/// An arbitrary name of any element, which additionally carries a span into the schema to mark its
/// location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Name<'a> {
    /// Raw string value.
    value: &'a str,
    /// Source code location.
    span: Span,
}

impl<'a> Name<'a> {
    /// Retrieve the raw string value of this name.
    #[must_use]
    pub const fn get(&self) -> &str {
        self.value
    }
}

impl<'a> Spanned for Name<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Display for Name<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'a> From<(&'a str, Range<usize>)> for Name<'a> {
    fn from((value, span): (&'a str, Range<usize>)) -> Self {
        Self {
            value,
            span: span.into(),
        }
    }
}

/// Declaration of a constant value.
#[derive(Debug, PartialEq)]
pub struct Const<'a> {
    /// Optional element-level comment.
    pub comment: Comment<'a>,
    /// Unique identifier of this constant.
    pub name: Name<'a>,
    /// Type of the value.
    pub ty: Type<'a>,
    /// Literal value that this declaration represents.
    pub value: Literal,
}

impl<'a> Print for Const<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            ty,
            value,
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        write!(f, "const {name}: {ty} = {value};")
    }
}

/// In-schema definition of a literal value.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// Boolean `true` or `false` value.
    Bool(bool),
    /// Integer number.
    Int(i128),
    /// Floating point number.
    Float(f64),
    /// UTF-8 encoded string.
    String(String),
    /// Raw vector of bytes.
    Bytes(Vec<u8>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Literal::Bool(v) => v.fmt(f),
            Literal::Int(v) => v.fmt(f),
            Literal::Float(v) => v.fmt(f),
            Literal::String(ref v) => write!(f, "{v:?}"),
            Literal::Bytes(ref v) => write!(f, "{v:?}"),
        }
    }
}

/// Import declaration for an external schema.
#[derive(Debug, PartialEq)]
pub struct Import<'a> {
    /// Individual elements that form the import path.
    pub segments: Vec<Name<'a>>,
    /// Optional final element that allows to fully import the type, making it look as it would be
    /// defined in the current schema.
    pub element: Option<Name<'a>>,
}

impl<'a> Print for Import<'a> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self { segments, element } = self;

        Self::indent(f, level)?;
        f.write_str("use ")?;

        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                f.write_str("::")?;
            }
            f.write_str(segment.get())?;
        }

        if let Some(element) = element {
            write!(f, "::{element}")?;
        }

        f.write_str(";")
    }
}

fn concat(
    f: &mut fmt::Formatter<'_>,
    open: &str,
    values: &[impl Display],
    sep: &str,
    close: &str,
) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }

    f.write_str(open)?;

    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        value.fmt(f)?;
    }

    f.write_str(close)
}
