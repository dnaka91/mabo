use cbindgen::{Builder, Config, DocumentationStyle, Language, RenameRule, SortKey};

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut config = Config::default();
    config.cpp_compat = true;
    config.documentation_style = DocumentationStyle::Doxy;
    config.enumeration.rename_variants = RenameRule::ScreamingSnakeCase;
    config.enumeration.prefix_with_name = true;
    config.include_guard = Some("MABO_CAPI_H".to_owned());
    config.language = Language::C;
    config.pragma_once = true;
    config.usize_is_size_t = true;
    config.sort_by=SortKey::Name;

    Builder::new()
        .with_config(config)
        .with_crate(crate_dir)
        .generate()
        .expect("unable to generate bindings")
        .write_to_file("bindings.h");
}
