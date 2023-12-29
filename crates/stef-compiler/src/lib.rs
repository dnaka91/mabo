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
use stef_parser::Spanned;
pub use validate::schema as validate_schema;

mod highlight;
pub mod resolve;
pub mod simplify;
pub mod validate;

/// Generator that is responsible for deriving omitted field and enum identifiers.
struct IdGenerator {
    /// The next available ID.
    next_id: u32,
}

impl IdGenerator {
    /// Create a new instance of the ID generator.
    fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Get the next ID, which is either already explicitly defined by the given parameter, or
    /// derived otherwise.
    ///
    /// Identifiers can have gaps, and must be consistently increasing (currently). In the future,
    /// the ID counter might jump forth and back, as long as each ID is guaranteed to be unique.
    fn next(&mut self, id: Option<&stef_parser::Id>) -> u32 {
        let id = id.map_or(self.next_id, stef_parser::Id::get);
        self.next_id = id + 1;
        id
    }

    /// Get the next ID, but additionally combine it with a span to construct a new
    /// [`stef_parser::Id`] instance.
    ///
    /// If an ID wasn't explicitly defined, the `fallback` closure is used to retrieve an
    /// alternative span. For example, this could be the span of a full struct field.
    fn next_with_span(
        &mut self,
        id: Option<&stef_parser::Id>,
        fallback: impl FnOnce() -> stef_parser::Span,
    ) -> stef_parser::Id {
        let new_id = self.next(id);
        let span = match id {
            Some(id) => id.span(),
            None => fallback(),
        };

        (new_id, span.into()).into()
    }
}
