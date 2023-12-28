//! Compiler for `STEF` schema files.
//!
//! In this context, compiling means to validate schema files and resolve all used types within.
//! This allows any code generator to consider its input valid and not having to do extra validation
//! work on the schemas.
//!
//! # Example
//!
//! Compile a basic `STEF` schema:
//!
//! ```
//! let schema = stef_parser::Schema::parse("struct Sample(u32 @1)", None).unwrap();
//!
//! // Ensure the schema in itself is valid (for example no duplicate IDs, type or field names).
//! stef_compiler::validate_schema(&schema).unwrap();
//! // Resolve all types used in the schema, both in the schema itself and its submodules, and in
//! // potentially types from external schemas that are referenced in it.
//! stef_compiler::resolve_schemas(&[("test", &schema)]).unwrap();
//! ```

#![allow(clippy::module_name_repetitions)]

pub use resolve::schemas as resolve_schemas;
pub use simplify::schema as simplify_schema;
pub use validate::schema as validate_schema;

mod highlight;
pub mod resolve;
pub mod simplify;
pub mod validate;
