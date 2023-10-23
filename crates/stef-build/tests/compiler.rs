use std::fs;

use insta::{assert_snapshot, glob};
use stef_parser::Schema;

#[test]
fn compile_schema() {
    glob!("inputs/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str()).unwrap();
        let value = stef_build::compile_schema(&value);
        let value = prettyplease::unparse(&syn::parse2(value.clone()).unwrap());

        assert_snapshot!("compile", format!("{value}"), input.trim());
    });
}

#[test]
fn compile_schema_extra() {
    glob!("inputs-extra/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str()).unwrap();
        let value = stef_build::compile_schema(&value);
        let value = prettyplease::unparse(&syn::parse2(value.clone()).unwrap());

        assert_snapshot!("compile-extra", format!("{value}"), input.trim());
    });
}
