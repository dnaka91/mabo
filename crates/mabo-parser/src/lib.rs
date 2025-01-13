//! Parser for Mabo schema files.
//!
//! The main entry point for schema parsing is the [`Schema::parse`] function. For convenience, the
//! [`from_str`] and [`from_slice`] functions are provided.
//!
//! # Example
//!
//! Parse a basic Mabo schema and print it back out.
//!
//! ```
//! let schema = mabo_parser::Schema::parse("struct Sample(u32 @1)", None).unwrap();
//!
//! // Pretty print the schema itself
//! println!("{schema}");
//! // Print the data structures themselves.
//! println!("{schema:#?}");
//! ```

use std::{
    fmt::{self, Display, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use mabo_derive::Debug;
pub use miette::{Diagnostic, LabeledSpan};
use miette::{IntoDiagnostic, NamedSource, Result};
use winnow::Parser;

use self::{error::ParseSchemaError, punctuated::Punctuated, token::Punctuation};
use crate::token::Delimiter;

pub mod error;
mod ext;
mod highlight;
mod location;
mod parser;
pub mod punctuated;
pub mod token;

/// Format trait like [`Display`], with the addition of indentation awareness.
pub trait Print {
    /// Default indentation, 4 spaces.
    const INDENT: &'static str = "    ";

    /// Write to the given formatter (like [`Display::fmt`]) but in addition, take the current
    /// indentation level into account.
    ///
    /// # Errors
    ///
    /// Will return `Err` if any of the formatting calls fails.
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result;

    /// Helper to write out the indentation for the given level.
    ///
    /// # Errors
    ///
    /// Will return `Err` if any of the formatting calls fails.
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

impl From<&Range<usize>> for Span {
    fn from(value: &Range<usize>) -> Self {
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

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
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

/// Uppermost element, describing a single Mabo Schema file.
#[derive(Debug, PartialEq)]
pub struct Schema<'a> {
    /// Physical location of the file that contains the schema source code.
    ///
    /// Might be missing if the schema originated from an inline string or otherwise omitted during
    /// the parsing process.
    pub path: Option<PathBuf>,
    /// Original source code form which this schema was parsed.
    pub source: &'a str,
    /// Optional schema-level comment.
    pub comment: Comment<'a>,
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
            .parse(winnow::LocatingSlice::new(input))
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

impl Display for Schema<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.comment.0.is_empty() {
            writeln!(f, "{}", self.comment)?;
        }

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

impl Print for Definition<'_> {
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

impl Display for Definition<'_> {
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
    /// The `mod` keyword to mark the module declaration.
    pub keyword: token::Mod,
    /// Unique name of the module, within the current scope.
    pub name: Name<'a>,
    /// Braces `{`...`}` around the contained definitions.
    pub brace: token::Brace,
    /// List of definitions that are scoped within this module.
    pub definitions: Vec<Definition<'a>>,
}

impl Print for Module<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            keyword,
            name,
            definitions,
            ..
        } = self;

        comment.print(f, level)?;
        keyword.print(f, level)?;

        writeln!(f, " {name} {}", token::Brace::OPEN)?;

        for (i, definition) in definitions.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            definition.print(f, level + 1)?;
        }

        Self::indent(f, level)?;
        writeln!(f, "{}", token::Brace::CLOSE)
    }
}

impl Display for Module<'_> {
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
    /// The `struct` keyword to mark the struct declaration.
    pub keyword: token::Struct,
    /// Unique name for this struct (within its scope).
    pub name: Name<'a>,
    /// Potential generics.
    pub generics: Option<Generics<'a>>,
    /// Fields of the struct, if any.
    pub fields: Fields<'a>,
}

impl Print for Struct<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            attributes,
            keyword,
            name,
            generics,
            fields: kind,
        } = self;

        comment.print(f, level)?;
        attributes.print(f, level)?;
        keyword.print(f, level)?;

        write!(f, " {name}")?;
        if let Some(generics) = generics {
            generics.fmt(f)?;
        }
        kind.print(f, level)?;

        f.write_str("\n")
    }
}

impl Display for Struct<'_> {
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
    /// The `enum` keyword to mark the enum declaration.
    pub keyword: token::Enum,
    /// Unique name for this enum, within its current scope.
    pub name: Name<'a>,
    /// Potential generics.
    pub generics: Option<Generics<'a>>,
    /// Braces `{`...`}` around the variants.
    pub brace: token::Brace,
    /// List of possible variants that the enum can represent.
    pub variants: Punctuated<Variant<'a>>,
}

impl Print for Enum<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            attributes,
            keyword,
            name,
            generics,
            variants,
            ..
        } = self;

        comment.print(f, level)?;
        attributes.print(f, level)?;
        keyword.print(f, level)?;

        write!(f, " {name}")?;
        if let Some(generics) = generics {
            generics.fmt(f)?;
        }

        f.write_char(' ')?;
        variants.surround::<token::Brace>(f, level, true)?;
        f.write_char('\n')
    }
}

impl Display for Enum<'_> {
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
    pub id: Option<Id>,
    /// Source code location.
    span: Span,
}

impl Print for Variant<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            fields,
            id,
            ..
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        f.write_str(name.get())?;
        fields.print(f, level)?;
        if let Some(id) = id {
            write!(f, " {id}")?;
        }
        Ok(())
    }
}

impl Spanned for Variant<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for Variant<'_> {
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
    /// Optional element-level comment.
    pub comment: Comment<'a>,
    /// The `type` keyword to mark the type alias declaration.
    pub keyword: token::Type,
    /// Unique name of the type alias within the current scope.
    pub name: Name<'a>,
    /// Potential generic type arguments.
    pub generics: Option<Generics<'a>>,
    /// Equal operator that assigns the target type.
    pub equal: token::Equal,
    /// Original type that is being aliased.
    pub target: Type<'a>,
    /// Trailing semicolon to complete the definition.
    pub semicolon: token::Semicolon,
}

impl Print for TypeAlias<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            keyword,
            name,
            generics,
            equal,
            target,
            semicolon,
        } = self;

        comment.print(f, level)?;
        keyword.print(f, level)?;

        write!(f, " {name}")?;
        if let Some(generics) = generics {
            generics.fmt(f)?;
        }
        write!(f, " {equal} {target}{semicolon}")
    }
}

impl Display for TypeAlias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Possible kinds in which the fields of a struct or enum variant can be represented.
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
    Named(token::Brace, Punctuated<NamedField<'a>>),
    /// List of types without an explicit name.
    ///
    /// ```txt
    /// Sample(u8 @1, bool @2, i32 @3)
    /// ```
    Unnamed(token::Parenthesis, Punctuated<UnnamedField<'a>>),
    /// No attached value.
    ///
    /// ```txt
    /// Sample
    /// ```
    Unit,
}

impl Print for Fields<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            Fields::Named(_, fields) => {
                f.write_char(' ')?;
                fields.surround::<token::Brace>(f, level, true)
            }
            Fields::Unnamed(_, elements) => elements.surround::<token::Parenthesis>(f, 0, false),
            Fields::Unit => Ok(()),
        }
    }
}

impl Display for Fields<'_> {
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
    /// Colon to separate the field name from the type.
    pub colon: token::Colon,
    /// Data type that defines the shape of the contained data.
    pub ty: Type<'a>,
    /// Identifier for this field, that must be unique within the current element.
    pub id: Option<Id>,
    /// Source code location.
    span: Span,
}

impl Print for NamedField<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            name,
            colon,
            ty,
            id,
            ..
        } = self;

        comment.print(f, level)?;

        Self::indent(f, level)?;
        write!(f, "{name}{colon} {ty}")?;

        if let Some(id) = id {
            write!(f, " {id}")?;
        }

        Ok(())
    }
}

impl Spanned for NamedField<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for NamedField<'_> {
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
    pub id: Option<Id>,
    /// Source code location.
    span: Span,
}

impl Spanned for UnnamedField<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Print for UnnamedField<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, _level: usize) -> fmt::Result {
        let Self { ty, id, .. } = self;
        write!(f, "{ty}")?;

        if let Some(id) = id {
            write!(f, " {id}")?;
        }

        Ok(())
    }
}

impl Display for UnnamedField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
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
pub struct Comment<'a>(pub Vec<CommentLine<'a>>);

impl Print for Comment<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let lines = &self.0;

        for line in lines {
            Self::indent(f, level)?;
            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

impl Display for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// Single [`Comment`] line, which additional tracks the location in the schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentLine<'a> {
    /// Raw string value.
    pub value: &'a str,
    /// Source code location (including the leading `/// ` marker).
    span: Span,
}

impl CommentLine<'_> {
    /// Retrieve the raw string value of this name.
    #[must_use]
    pub const fn get(&self) -> &str {
        self.value
    }
}

impl Spanned for CommentLine<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for CommentLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/// {}", self.value)
    }
}

impl<'a> From<(&'a str, Range<usize>)> for CommentLine<'a> {
    fn from((value, span): (&'a str, Range<usize>)) -> Self {
        Self {
            value,
            span: span.into(),
        }
    }
}

impl AsRef<str> for CommentLine<'_> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

/// Collection of attributes, aggregated together into a single declaration block.
#[derive(Debug, Default, PartialEq)]
pub struct Attributes<'a>(pub Vec<Attribute<'a>>);

impl Print for Attributes<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        let values = &self.0;

        Self::indent(f, level)?;
        f.write_str(token::Pound::VALUE)?;
        concat::<token::Bracket>(f, values, ", ")?;
        f.write_char('\n')
    }
}

impl Display for Attributes<'_> {
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

impl Print for Attribute<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let indent = Self::INDENT.repeat(level);
        let Self { name, value } = self;

        write!(f, "{indent}{name}{value}")
    }
}

impl Display for Attribute<'_> {
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

impl Print for AttributeValue<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, _level: usize) -> fmt::Result {
        match self {
            Self::Unit => Ok(()),
            Self::Single(lit) => write!(f, " = {lit}"),
            Self::Multi(attrs) => concat::<token::Parenthesis>(f, attrs, ", "),
        }
    }
}

impl Display for AttributeValue<'_> {
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

impl Spanned for Type<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Print for Type<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, _level: usize) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
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
    Vec {
        /// Source code location of the `vec` type name.
        span: Span,
        /// Angles `<`...`>` that delimit the type parameter.
        angle: token::Angle,
        /// Type parameter.
        ty: Box<Type<'a>>,
    },
    /// Key-value hash map of data types.
    HashMap {
        /// Source code location of the `hash_map` type name.
        span: Span,
        /// Angles `<`...`>` that delimit the type parameters.
        angle: token::Angle,
        /// First type parameter.
        key: Box<Type<'a>>,
        /// Separator between first and second type parameter.
        comma: token::Comma,
        /// Second type parameter.
        value: Box<Type<'a>>,
    },
    /// Hash set of data types (each entry is unique).
    HashSet {
        /// Source code location of the `hash_set` type name.
        span: Span,
        /// Angles `<`...`>` that delimit the type parameter.
        angle: token::Angle,
        /// Type parameter.
        ty: Box<Type<'a>>,
    },
    /// Optional value.
    Option {
        /// Source code location of the `option` type name.
        span: Span,
        /// Angles `<`...`>` that delimit the type parameter.
        angle: token::Angle,
        /// Type parameter.
        ty: Box<Type<'a>>,
    },
    /// Non-zero value.
    /// - Integers: `n > 0`
    /// - Collections: `len() > 0`
    NonZero {
        /// Source code location of the `non_zero` type name.
        span: Span,
        /// Angles `<`...`>` that delimit the type parameter.
        angle: token::Angle,
        /// Type parameter.
        ty: Box<Type<'a>>,
    },
    /// Boxed version of a string that is immutable.
    BoxString,
    /// Boxed version of a byte vector that is immutable.
    BoxBytes,
    /// Fixed size list of up to 12 types.
    Tuple {
        /// Parenthesis `(`...`)` that delimits the tuple.
        paren: token::Parenthesis,
        /// List of types that make up the tuple.
        types: Punctuated<Type<'a>>,
    },
    /// Continuous list of values with a single time and known length.
    Array {
        /// Brackets `[`...`]` that delimit the array.
        bracket: token::Bracket,
        /// The singular repeated type.
        ty: Box<Type<'a>>,
        /// Separator between type and array size.
        semicolon: token::Semicolon,
        /// Size, as in count of elements.
        size: u32,
    },
    /// Any external, non-standard data type (like a user defined struct or enum).
    External(ExternalType<'a>),
}

impl Display for DataType<'_> {
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
            Self::Vec { ty, .. } => write!(f, "vec<{ty}>"),
            Self::HashMap { key, value, .. } => write!(f, "hash_map<{key}, {value}>"),
            Self::HashSet { ty, .. } => write!(f, "hash_set<{ty}>"),
            Self::Option { ty, .. } => write!(f, "option<{ty}>"),
            Self::NonZero { ty, .. } => write!(f, "non_zero<{ty}>"),
            Self::BoxString => f.write_str("box<string>"),
            Self::BoxBytes => f.write_str("box<bytes>"),
            Self::Tuple { types, .. } => types.surround::<token::Parenthesis>(f, 0, false),
            Self::Array { ty, size, .. } => write!(f, "[{ty}; {size}]"),
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
    pub path: Vec<(Name<'a>, token::DoubleColon)>,
    /// Unique name of the type within the current scope (or the module if prefixed with a path).
    pub name: Name<'a>,
    /// Angles `<`...`>` to delimit the generic type parameters.
    pub angle: Option<token::Angle>,
    /// Potential generic type arguments.
    pub generics: Option<Punctuated<Type<'a>>>,
}

impl Display for ExternalType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            path,
            name,
            generics,
            ..
        } = self;

        for (segment, token) in path {
            write!(f, "{segment}{token}")?;
        }
        name.fmt(f)?;
        if let Some(generics) = generics {
            generics.surround::<token::Angle>(f, 0, false)?;
        }
        Ok(())
    }
}

/// Container of generic arguments for an element.
///
/// ```txt
/// <A, B, ...>
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Generics<'a> {
    /// Angles `<`...`>` to delimit the generic type parameters.
    pub angle: token::Angle,
    /// The generic types, separated by commas `,`.
    pub types: Punctuated<Name<'a>>,
}

impl Display for Generics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.types.surround::<token::Angle>(f, 0, false)
    }
}

/// Unique identifier for an element.
///
/// ```txt
/// @1
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

impl From<(u32, Range<usize>)> for Id {
    fn from((value, span): (u32, Range<usize>)) -> Self {
        Self {
            value,
            span: span.into(),
        }
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

impl Name<'_> {
    /// Retrieve the raw string value of this name.
    #[must_use]
    pub const fn get(&self) -> &str {
        self.value
    }
}

impl Spanned for Name<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Print for Name<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, _level: usize) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
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

impl AsRef<str> for Name<'_> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

/// Declaration of a constant value.
#[derive(Debug, PartialEq)]
pub struct Const<'a> {
    /// Optional element-level comment.
    pub comment: Comment<'a>,
    /// The `const` keyword to mark the constant declaration.
    pub keyword: token::Const,
    /// Unique identifier of this constant.
    pub name: Name<'a>,
    /// Colon to separate the const name from the type.
    pub colon: token::Colon,
    /// Type of the value.
    pub ty: Type<'a>,
    /// Equal operator that assigns the literal value.
    pub equal: token::Equal,
    /// Literal value that this declaration represents.
    pub value: Literal,
    /// Trailing semicolon to complete the definition.
    pub semicolon: token::Semicolon,
}

impl Print for Const<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            comment,
            keyword,
            name,
            colon,
            ty,
            equal,
            value,
            semicolon,
        } = self;

        comment.print(f, level)?;
        keyword.print(f, level)?;

        write!(f, " {name}{colon} {ty} {equal} {value}{semicolon}")
    }
}

impl Display for Const<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print(f, 0)
    }
}

/// In-schema definition of a literal value, together with a span into the schema to mark where it
/// is defined.
#[derive(Clone, Debug, PartialEq)]
pub struct Literal {
    /// The raw literal value.
    pub value: LiteralValue,
    /// Source code location.
    span: Span,
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        self.span
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl From<(LiteralValue, Range<usize>)> for Literal {
    fn from((value, span): (LiteralValue, Range<usize>)) -> Self {
        Self {
            value,
            span: span.into(),
        }
    }
}

/// Raw value of a [`Literal`].
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
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

impl Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Bool(v) => v.fmt(f),
            Self::Int(v) => v.fmt(f),
            Self::Float(v) => v.fmt(f),
            Self::String(ref v) => write!(f, "{v:?}"),
            Self::Bytes(ref v) => write!(f, "{v:?}"),
        }
    }
}

/// Import declaration for an external schema.
#[derive(Debug, PartialEq)]
pub struct Import<'a> {
    /// The `use` keyword to mark the import declaration.
    pub keyword: token::Use,
    /// Full import path as it was found in the original schema file.
    pub full: Name<'a>,
    /// Individual elements that form the import path.
    pub segments: Vec<Name<'a>>,
    /// Optional final element that allows to fully import the type, making it look as it would be
    /// defined in the current schema.
    pub element: Option<(token::DoubleColon, Name<'a>)>,
    /// Trailing semicolon to complete the definition.
    pub semicolon: token::Semicolon,
}

impl Print for Import<'_> {
    fn print(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        let Self {
            keyword,
            segments,
            element,
            semicolon,
            ..
        } = self;

        keyword.print(f, level)?;
        f.write_str(" ")?;

        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                f.write_str("::")?;
            }
            f.write_str(segment.get())?;
        }

        if let Some((token, element)) = element {
            write!(f, "{token}{element}")?;
        }

        write!(f, "{semicolon}")
    }
}

fn concat<D: Delimiter>(
    f: &mut fmt::Formatter<'_>,
    values: &[impl Display],
    sep: &str,
) -> fmt::Result {
    if values.is_empty() {
        return Ok(());
    }

    f.write_char(D::OPEN)?;

    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        value.fmt(f)?;
    }

    f.write_char(D::CLOSE)
}
