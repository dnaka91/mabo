use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use insta::{assert_snapshot, glob, with_settings};
use stef_go::{Opts, Output};
use stef_parser::Schema;

fn strip_path(path: &Path) -> PathBuf {
    path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs"))
        .or_else(|_| path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs_extra")))
        .unwrap()
        .to_owned()
}

fn merge_output(buf: &mut String, output: Output<'_>, parent: &Path) {
    let path = parent.join(output.name);
    let _ = write!(buf, "--- {}.go\n\n{}", path.display(), output.content);

    for module in output.modules {
        merge_output(buf, module, &path);
    }
}

#[test]
fn render_schema() {
    glob!("inputs/*.stef", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let value = stef_go::render_schema(&Opts { package: "sample" }, &value);

        let mut merged = String::new();
        merge_output(&mut merged, value, Path::new(""));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("render", merged);
        });
    });
}
