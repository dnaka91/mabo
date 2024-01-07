#![allow(missing_docs)]

fn main() -> mabo_build::Result<()> {
    mabo_build::Compiler::default().compile(env!("CARGO_MANIFEST_DIR"))
}
