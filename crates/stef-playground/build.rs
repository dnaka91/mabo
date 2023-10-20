fn main() -> anyhow::Result<()> {
    stef_build::compile(&["src/sample.stef"], &["src/"])?;
    stef_build::compile(&["schemas/*.stef"], &["schemas/"])?;
    Ok(())
}
