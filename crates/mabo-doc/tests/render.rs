use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use insta::{assert_snapshot, glob, with_settings};
use mabo_doc::{Opts, Output};
use mabo_parser::Schema;

fn strip_path(path: &Path) -> PathBuf {
    path.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs"))
        .unwrap()
        .to_owned()
}

fn merge_output(buf: &mut String, output: &Output<'_>, parent: &Path) {
    let path = parent.join(output.name);
    let file = parent.join(&output.file);
    let _ = write!(buf, "\n\n--- {}\n\n{}", file.display(), output.content);

    for module in &output.modules {
        merge_output(buf, module, &path);
    }
}

#[test]
fn render_schema() {
    glob!("inputs/*.mabo", |path| {
        let input = fs::read_to_string(path).unwrap();
        let value = Schema::parse(input.as_str(), Some(&strip_path(path))).unwrap();
        let value = mabo_compiler::simplify_schema(&value);
        let value = mabo_doc::render_schema(&Opts {}, &value).unwrap();

        let mut merged = String::new();
        merge_output(&mut merged, &value, Path::new(""));

        with_settings!({
            description => input.trim(),
            omit_expression => true,
        }, {
            assert_snapshot!("render", merged.trim_start());
        });
    });
}
