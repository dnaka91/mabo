use std::{
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
};

use insta::{assert_snapshot, glob, with_settings};
use miette::{Diagnostic, MietteHandler, MietteHandlerOpts, ReportHandler};
use mabo_parser::Schema;

fn strip_path(path: &Path) -> PathBuf {
    path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs"))
        .unwrap()
        .to_owned()
}

#[test]
fn parse_schema() {
    glob!("inputs/*.mabo", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("parse", format!("{value:#?}"));
            assert_snapshot!("print", value.to_string());
        });
    });
}

#[test]
fn parse_invalid_schema() {
    struct Wrapper<'a>(&'a MietteHandler, &'a dyn Diagnostic);

    impl Display for Wrapper<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.debug(self.1, f)
        }
    }

    let handler = MietteHandlerOpts::new()
        .color(false)
        .terminal_links(false)
        .width(120)
        .force_graphical(true)
        .build();

    glob!("inputs/invalid/*.mabo", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap_err();

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error", Wrapper(&handler, &value).to_string());
        });
    });
}
