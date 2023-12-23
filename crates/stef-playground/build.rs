#![allow(missing_docs)]

fn main() -> stef_build::Result<()> {
    let compiler = stef_build::Compiler::default();
    compiler.compile(&["src/evolution.stef", "src/sample.stef"])?;
    compiler.compile(&["schemas/*.stef", "src/other.stef", "src/second.stef"])?;
    Ok(())
}
