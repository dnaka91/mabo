use std::fs;

use insta::{assert_snapshot, glob, with_settings};
use stef_parser::Schema;

#[test]
fn compile_schema() {
    glob!("inputs/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str()).unwrap();
        let value = stef_build::compile_schema(&value);
        let value = prettyplease::unparse(&syn::parse2(value.clone()).unwrap());

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("compile", value);
        });
    });
}

#[test]
fn compile_schema_extra() {
    glob!("inputs_extra/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str()).unwrap();
        let value = stef_build::compile_schema(&value);
        let value = prettyplease::unparse(&syn::parse2(value.clone()).unwrap());

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("compile_extra", value);
        });
    });
}
