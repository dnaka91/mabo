use std::{
    fmt::{self, Display},
    fs,
};

use insta::{assert_snapshot,with_settings, glob};
use miette::{Diagnostic, MietteHandler, MietteHandlerOpts, NamedSource, Report, ReportHandler};
use stef_parser::Schema;

#[test]
fn compile_invalid_schema() {
    struct Wrapper<'a>(&'a MietteHandler, &'a dyn Diagnostic);

    impl<'a> Display for Wrapper<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.debug(self.1, f)
        }
    }

    let handler = MietteHandlerOpts::new()
        .color(false)
        .context_lines(3)
        .force_graphical(true)
        .terminal_links(false)
        .width(120)
        .build();

    glob!("inputs/invalid/*.stef", |path| {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let input = fs::read_to_string(path).unwrap();
        let schema = Schema::parse(input.as_str()).unwrap();
        let result = stef_compiler::validate_schema(name, &schema).unwrap_err();
        let report = Report::new(result).with_source_code(NamedSource::new(
            path.file_name().unwrap().to_string_lossy(),
            input.clone(),
        ));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("error", Wrapper(&handler, &*report).to_string());
        });
    });
}
