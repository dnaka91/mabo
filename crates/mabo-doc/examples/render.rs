//! Helper that renders all the test snapshot inputs as HTML files out into the OS's temp folder.
//!
//! All required assets are copied to the right location as well, so each input file is turned into
//! an atomic output folder, as if it would be run through the CLI in the future (similar to
//! `rustdoc` being invoked by `cargo doc`).

use std::{env, fs, path::Path};

use mabo_doc::{Opts, Output};
use mabo_parser::Schema;

fn write_output(output: &Output<'_>, parent: &Path) {
    let path = parent.join(output.name);
    let file = parent.join(&output.file);

    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, &output.content).unwrap();

    for module in &output.modules {
        write_output(module, &path);
    }
}

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/inputs");
    let out = env::temp_dir().join(concat!("mabo/", env!("CARGO_PKG_NAME")));

    for path in glob::glob(&format!("{dir}/*.mabo")).unwrap() {
        let path = path.unwrap();
        let name = path.strip_prefix(dir).unwrap();

        let input = fs::read_to_string(&path).unwrap();
        let value = Schema::parse(input.as_str(), Some(name)).unwrap();
        let value = mabo_compiler::simplify_schema(&value);
        let value = mabo_doc::render_schema(&Opts {}, &value).unwrap();

        let out = out.join(name).with_extension("");

        fs::remove_dir_all(&out).ok();
        fs::create_dir_all(out.join("assets")).unwrap();
        fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/style.css"),
            out.join("assets/style.css"),
        )
        .unwrap();

        write_output(&value, &out);
    }
}
