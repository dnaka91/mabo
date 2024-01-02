use std::{
    fs,
    path::{Path, PathBuf},
};

use insta::{assert_snapshot, glob, with_settings};
use mabo_build::Opts;
use mabo_parser::Schema;

fn strip_path(path: &Path) -> PathBuf {
    path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs"))
        .or_else(|_| path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs_extra")))
        .unwrap()
        .to_owned()
}

#[test]
fn compile_schema() {
    glob!("inputs/*.mabo", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let value = mabo_compiler::simplify_schema(&value);
        let value = mabo_build::compile_schema(&Opts::default(), &value);
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
    glob!("inputs_extra/*.mabo", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let value = mabo_compiler::simplify_schema(&value);
        let value = mabo_build::compile_schema(&Opts::default(), &value);
        let value = prettyplease::unparse(&syn::parse2(value.clone()).unwrap());

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("compile_extra", value);
        });
    });
}
