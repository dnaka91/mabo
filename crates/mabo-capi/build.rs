#![allow(missing_docs)]

use cbindgen::{
    Builder, Config, DocumentationStyle, EnumConfig, ExportConfig, Language, RenameRule, SortKey,
    Style,
};

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let config = Config {
        include_guard: Some("MABO_CAPI_H".to_owned()),
        pragma_once: true,
        tab_width: 4,
        language: Language::C,
        cpp_compat: true,
        style: Style::Both,
        sort_by: SortKey::Name,
        usize_is_size_t: false,
        enumeration: EnumConfig {
            rename_variants: RenameRule::ScreamingSnakeCase,
            prefix_with_name: true,
            ..EnumConfig::default()
        },
        documentation_style: DocumentationStyle::Doxy,
        export: ExportConfig {
            prefix: Some("Mabo".to_owned()),
            ..ExportConfig::default()
        },
        ..Config::default()
    };

    Builder::new()
        .with_config(config)
        .with_crate(crate_dir)
        .generate()
        .expect("unable to generate bindings")
        .write_to_file("bindings.h");
}
