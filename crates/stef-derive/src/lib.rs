#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::too_many_lines
)]

use syn::{parse_macro_input, DeriveInput};

mod attributes;
mod cause;
mod debug;
mod error;

#[proc_macro_derive(ParserError, attributes(err, rename))]
pub fn parser_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match error::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(ParserErrorCause, attributes(err, external, forward, rename))]
pub fn parser_error_cause(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match cause::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(Debug)]
pub fn debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match debug::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
