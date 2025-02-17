//! Internal derive macros for several Mabo crates, that reduce boilerplate or create more
//! specialized implementations that the stdlib derives.

#![expect(clippy::too_many_lines)]

use syn::{DeriveInput, parse_macro_input};

mod attributes;
mod cause;
mod debug;
mod error;

/// /// Derive the [`miette`](https://docs.rs/miette) and [`winnow`](https://docs.rs/winnow) traits for
/// an error struct that is coupled with a cause enum.
#[proc_macro_derive(ParserError, attributes(err, rename))]
pub fn parser_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match error::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Derive the [`miette`](https://docs.rs/miette) and [`winnow`](https://docs.rs/winnow) traits for
/// an error cause enum, which contains one of the possible causes for a failure in parsing.
///
/// The first variant of any enum must be named _Parser_, and contain two unnamed fields with type
/// `ErrorKind` and `usize`. This variant catches generic parser errors from `winnow` and their
/// location.
#[proc_macro_derive(ParserErrorCause, attributes(err, external, forward, rename))]
pub fn parser_error_cause(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match cause::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Specialized [`core::fmt::Debug`] macro, which omits span fields from the output.
#[proc_macro_derive(Debug)]
pub fn debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match debug::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
