use std::{
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use insta::{assert_snapshot, glob, with_settings};
use miette::{Diagnostic, MietteHandler, MietteHandlerOpts, NamedSource, Report, ReportHandler};
use stef_parser::Schema;

struct Wrapper<'a>(&'a MietteHandler, &'a dyn Diagnostic);

impl<'a> Wrapper<'a> {
    fn new(diagnostic: &'a dyn Diagnostic) -> Self {
        static HANDLER: OnceLock<MietteHandler> = OnceLock::new();

        Self(
            HANDLER.get_or_init(|| {
                MietteHandlerOpts::new()
                    .color(false)
                    .context_lines(3)
                    .force_graphical(true)
                    .terminal_links(false)
                    .width(120)
                    .build()
            }),
            diagnostic,
        )
    }
}

impl<'a> Display for Wrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug(self.1, f)
    }
}

fn strip_path(path: &Path) -> PathBuf {
    path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs"))
        .unwrap()
        .to_owned()
}

#[test]
fn validate_schema() {
    glob!("inputs/validate/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let schema = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let result = stef_compiler::validate_schema(&schema).unwrap_err();
        let report = Report::new(result).with_source_code(NamedSource::new(
            path.file_name().unwrap().to_string_lossy(),
            input.clone(),
        ));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error_validate", Wrapper::new(&*report).to_string());
        });
    });
}

#[test]
fn resolve_schema_local() {
    glob!("inputs/resolve/local_*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let schema = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let result = stef_compiler::resolve_schemas(&[("test", &schema)]).unwrap_err();
        let report = Report::new(result).with_source_code(NamedSource::new(
            path.file_name().unwrap().to_string_lossy(),
            input.clone(),
        ));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error_resolve", Wrapper::new(&*report).to_string());
        });
    });
}

#[test]
fn resolve_schema_import() {
    let input = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/inputs/resolve/datetime.stef"
    ));
    let datetime = Schema::parse(input, Some(Path::new("resolve/datetime.stef"))).unwrap();

    glob!("inputs/resolve/import_*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let schema = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let result = stef_compiler::resolve_schemas(&[("test", &schema), ("datetime", &datetime)])
            .unwrap_err();
        let report = Report::new(result).with_source_code(NamedSource::new(
            path.file_name().unwrap().to_string_lossy(),
            input.clone(),
        ));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error_resolve", Wrapper::new(&*report).to_string());
        });
    });
}

#[test]
fn resolve_schema_remote() {
    let input = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/inputs/resolve/datetime.stef"
    ));
    let datetime = Schema::parse(input, Some(Path::new("resolve/datetime.stef"))).unwrap();

    glob!("inputs/resolve/remote_*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let schema = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let result = stef_compiler::resolve_schemas(&[("test", &schema), ("datetime", &datetime)])
            .unwrap_err();
        let report = Report::new(result).with_source_code(NamedSource::new(
            path.file_name().unwrap().to_string_lossy(),
            input.clone(),
        ));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error_resolve", Wrapper::new(&*report).to_string());
        });
    });
}
