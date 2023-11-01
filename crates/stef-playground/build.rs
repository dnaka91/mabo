fn main() -> stef_build::Result<()> {
    stef_build::compile(&["src/sample.stef"], &["src/"])?;
    stef_build::compile(
        &["schemas/*.stef", "src/other.stef", "src/second.stef"],
        &["schemas/"],
    )?;
    Ok(())
}
