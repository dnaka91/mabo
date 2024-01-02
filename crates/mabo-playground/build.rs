#![allow(missing_docs)]

fn main() -> mabo_build::Result<()> {
    let compiler = mabo_build::Compiler::default();
    compiler.compile(&["src/evolution.mabo", "src/sample.mabo"])?;
    compiler.compile(&["schemas/*.mabo", "src/other.mabo", "src/second.mabo"])?;
    Ok(())
}
