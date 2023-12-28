//! Reduce the complexity of the schema types that are used by the parser, making it easier for code
//! generators to operate on and reducing the amount of duplicate calculations that can be done in
//! the compiler instead.

use std::borrow::Cow;

/// Uppermost element, describing a single schema file.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Schema<'a> {
    /// Optional schema-level comment.
    pub comment: Box<[&'a str]>,
    /// List of all the definitions that make up the schema.
    pub definitions: Box<[Definition<'a>]>,
}

impl Schema<'_> {
    /// Render the [JSON Schema](https://json-schema.org/draft-07/json-schema-release-notes) of the complete schema, which can help external tools to understand the structure or derive types from it.
    #[cfg(feature = "json")]
    pub fn json_schema() -> serde_json::Result<String> {
        let schema = schemars::schema_for!(Schema<'_>);
        serde_json::to_string_pretty(&schema)
    }
}

/// Possible elements that can appear inside a [`Schema`] or [`Module`].
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
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

/// Scoping mechanism to categorize elements.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Module<'a> {
    /// Optional module-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique name of the module, within the current scope.
    pub name: &'a str,
    /// List of definitions that are scoped within this module.
    pub definitions: Box<[Definition<'a>]>,
}

/// Rust-ish struct.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Struct<'a> {
    /// Optional struct-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique name for this struct (within its scope).
    pub name: &'a str,
    /// Potential generics.
    pub generics: Box<[&'a str]>,
    /// Fields of the struct, if any.
    pub fields: Fields<'a>,
}

/// Rust-ish enum.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Enum<'a> {
    /// Optional enum-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique name for this enum, within its current scope.
    pub name: &'a str,
    /// Potential generics.
    pub generics: Box<[&'a str]>,
    /// List of possible variants that the enum can represent.
    pub variants: Vec<Variant<'a>>,
}

/// Single variant of an enum.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Variant<'a> {
    /// Optional variant-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique for this variant, within the enum it belongs to.
    pub name: &'a str,
    /// Fields of this variant, if any.
    pub fields: Fields<'a>,
    /// Identifier for this variant, that must be unique within the current enum.
    pub id: u32,
}

/// Fields of a struct or enum that define its structure.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Fields<'a> {
    /// List of contained fields.
    pub fields: Box<[Field<'a>]>,
    /// The way how the fields are defined, like named or unnamed.
    pub kind: FieldKind,
}

/// Single unified field that might be named or unnamed.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Field<'a> {
    /// Optional field-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique name for this field, within the current element.
    pub name: Cow<'a, str>,
    /// Data type that defines the shape of the contained data.
    pub ty: Type<'a>,
    /// Identifier for this field, that must be unique within the current element.
    pub id: u32,
}

/// Possible kinds in which the fields of a struct or enum variant can be represented.
#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub enum FieldKind {
    /// Named fields.
    Named,
    /// Types without an explicit name.
    Unnamed,
    /// No attached value.
    Unit,
}

/// Alias (re-name) from one type to another.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct TypeAlias<'a> {
    /// Optional element-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique name of the type alias within the current scope.
    pub name: &'a str,
    /// Potential generic type arguments.
    pub generics: Box<[&'a str]>,
    /// Original type that is being aliased.
    pub target: Type<'a>,
}

/// Declaration of a constant value.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Const<'a> {
    /// Optional element-level comment.
    pub comment: Box<[&'a str]>,
    /// Unique identifier of this constant.
    pub name: &'a str,
    /// Type of the value.
    pub ty: Type<'a>,
    /// Literal value that this declaration represents.
    pub value: Literal,
}

/// In-schema definition of a literal value
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub enum Literal {
    /// Boolean `true` or `false` value.
    Bool(bool),
    /// Integer number.
    Int(i128),
    /// Floating point number.
    Float(f64),
    /// UTF-8 encoded string.
    String(Box<str>),
    /// Raw vector of bytes.
    Bytes(Box<[u8]>),
}

/// Import declaration for an external schema.
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct Import<'a> {
    /// Individual elements that form the import path.
    pub segments: Box<[&'a str]>,
    /// Optional final element that allows to fully import the type, making it look as it would be
    /// defined in the current schema.
    pub element: Option<Box<str>>,
}

/// Possible data type that describes the shape of a field.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub enum Type<'a> {
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
    Tuple(Box<[Type<'a>]>),
    /// Continuous list of values with a single time and known length.
    Array(Box<Type<'a>>, u32),
    /// Any external, non-standard data type (like a user defined struct or enum).
    External(ExternalType<'a>),
}

/// Type that is not part of the built-in list of types.
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json", derive(schemars::JsonSchema, serde::Serialize))]
pub struct ExternalType<'a> {
    /// Optional path, if the type wasn't fully imported with a `use` statement.
    pub path: Box<[&'a str]>,
    /// Unique name of the type within the current scope (or the module if prefixed with a path).
    pub name: &'a str,
    /// Potential generic type arguments.
    pub generics: Vec<Type<'a>>,
}

/// Transform the schema into a simpler form, which has less but still enough details to generate
/// language implementations for a schema.
#[must_use]
pub fn schema<'a>(schema: &'a stef_parser::Schema<'_>) -> Schema<'a> {
    Schema {
        comment: comment(&schema.comment),
        definitions: definitions(&schema.definitions),
    }
}

#[inline]
fn comment<'a>(item: &'a stef_parser::Comment<'_>) -> Box<[&'a str]> {
    item.0.iter().map(|line| line.value).collect()
}

#[inline]
fn generics<'a>(item: &'a stef_parser::Generics<'_>) -> Box<[&'a str]> {
    item.0.iter().map(stef_parser::Name::get).collect()
}

#[inline]
fn definitions<'a>(item: &'a [stef_parser::Definition<'_>]) -> Box<[Definition<'a>]> {
    item.iter().map(|def| definition(def)).collect()
}

fn definition<'a>(item: &'a stef_parser::Definition<'_>) -> Definition<'a> {
    match item {
        stef_parser::Definition::Module(m) => Definition::Module(simplify_module(m)),
        stef_parser::Definition::Struct(s) => Definition::Struct(simplify_struct(s)),
        stef_parser::Definition::Enum(e) => Definition::Enum(simplify_enum(e)),
        stef_parser::Definition::TypeAlias(a) => Definition::TypeAlias(simplify_alias(a)),
        stef_parser::Definition::Const(c) => Definition::Const(simplify_const(c)),
        stef_parser::Definition::Import(i) => Definition::Import(simplify_import(i)),
    }
}

fn simplify_module<'a>(item: &'a stef_parser::Module<'_>) -> Module<'a> {
    Module {
        comment: comment(&item.comment),
        name: item.name.get(),
        definitions: definitions(&item.definitions),
    }
}

fn simplify_struct<'a>(item: &'a stef_parser::Struct<'_>) -> Struct<'a> {
    Struct {
        comment: comment(&item.comment),
        name: item.name.get(),
        generics: generics(&item.generics),
        fields: simplify_fields(&item.fields),
    }
}

fn simplify_enum<'a>(item: &'a stef_parser::Enum<'_>) -> Enum<'a> {
    Enum {
        comment: comment(&item.comment),
        name: item.name.get(),
        generics: generics(&item.generics),
        variants: item
            .variants
            .iter()
            .map(|variant| simplify_variant(variant))
            .collect(),
    }
}

fn simplify_variant<'a>(item: &'a stef_parser::Variant<'_>) -> Variant<'a> {
    Variant {
        comment: comment(&item.comment),
        name: item.name.get(),
        fields: simplify_fields(&item.fields),
        id: item.id.get(),
    }
}

fn simplify_fields<'a>(item: &'a stef_parser::Fields<'_>) -> Fields<'a> {
    match item {
        stef_parser::Fields::Named(named) => Fields {
            fields: named
                .iter()
                .map(|field| Field {
                    comment: comment(&field.comment),
                    name: field.name.get().into(),
                    ty: simplify_type(&field.ty),
                    id: field.id.get(),
                })
                .collect(),
            kind: FieldKind::Named,
        },
        stef_parser::Fields::Unnamed(unnamed) => Fields {
            fields: unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| Field {
                    comment: Box::default(),
                    name: format!("n{i}").into(),
                    ty: simplify_type(&field.ty),
                    id: field.id.get(),
                })
                .collect(),
            kind: FieldKind::Unnamed,
        },
        stef_parser::Fields::Unit => Fields {
            fields: Box::default(),
            kind: FieldKind::Unit,
        },
    }
}

fn simplify_type<'a>(item: &'a stef_parser::Type<'_>) -> Type<'a> {
    match item.value {
        stef_parser::DataType::Bool => Type::Bool,
        stef_parser::DataType::U8 => Type::U8,
        stef_parser::DataType::U16 => Type::U16,
        stef_parser::DataType::U32 => Type::U32,
        stef_parser::DataType::U64 => Type::U64,
        stef_parser::DataType::U128 => Type::U128,
        stef_parser::DataType::I8 => Type::I8,
        stef_parser::DataType::I16 => Type::I16,
        stef_parser::DataType::I32 => Type::I32,
        stef_parser::DataType::I64 => Type::I64,
        stef_parser::DataType::I128 => Type::I128,
        stef_parser::DataType::F32 => Type::F32,
        stef_parser::DataType::F64 => Type::F64,
        stef_parser::DataType::String => Type::String,
        stef_parser::DataType::StringRef => Type::StringRef,
        stef_parser::DataType::Bytes => Type::Bytes,
        stef_parser::DataType::BytesRef => Type::BytesRef,
        stef_parser::DataType::Vec(ref ty) => Type::Vec(simplify_type(ty).into()),
        stef_parser::DataType::HashMap(ref kv) => {
            Type::HashMap((simplify_type(&kv.0), simplify_type(&kv.1)).into())
        }
        stef_parser::DataType::HashSet(ref ty) => Type::HashSet(simplify_type(ty).into()),
        stef_parser::DataType::Option(ref ty) => Type::Option(simplify_type(ty).into()),
        stef_parser::DataType::NonZero(ref ty) => Type::NonZero(simplify_type(ty).into()),
        stef_parser::DataType::BoxString => Type::BoxString,
        stef_parser::DataType::BoxBytes => Type::BoxBytes,
        stef_parser::DataType::Tuple(ref types) => {
            Type::Tuple(types.iter().map(|ty| simplify_type(ty)).collect())
        }
        stef_parser::DataType::Array(ref ty, size) => Type::Array(simplify_type(ty).into(), size),
        stef_parser::DataType::External(ref ty) => Type::External(ExternalType {
            path: ty.path.iter().map(stef_parser::Name::get).collect(),
            name: ty.name.get(),
            generics: ty.generics.iter().map(|ty| simplify_type(ty)).collect(),
        }),
    }
}

fn simplify_alias<'a>(item: &'a stef_parser::TypeAlias<'_>) -> TypeAlias<'a> {
    TypeAlias {
        comment: comment(&item.comment),
        name: item.name.get(),
        generics: generics(&item.generics),
        target: simplify_type(&item.target),
    }
}

fn simplify_const<'a>(item: &'a stef_parser::Const<'_>) -> Const<'a> {
    Const {
        comment: comment(&item.comment),
        name: item.name.get(),
        ty: simplify_type(&item.ty),
        value: simplify_literal(&item.value),
    }
}

fn simplify_literal(item: &stef_parser::Literal) -> Literal {
    match item.value {
        stef_parser::LiteralValue::Bool(b) => Literal::Bool(b),
        stef_parser::LiteralValue::Int(i) => Literal::Int(i),
        stef_parser::LiteralValue::Float(f) => Literal::Float(f),
        stef_parser::LiteralValue::String(ref s) => Literal::String(s.clone().into()),
        stef_parser::LiteralValue::Bytes(ref b) => Literal::Bytes(b.clone().into()),
    }
}

fn simplify_import<'a>(item: &'a stef_parser::Import<'_>) -> Import<'a> {
    Import {
        segments: item.segments.iter().map(stef_parser::Name::get).collect(),
        element: item.element.as_ref().map(|element| element.get().into()),
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "json")]
    #[test]
    fn json_schema() {
        super::Schema::json_schema().unwrap();
    }
}
