use anyhow::{ensure, Context, Result};
use line_index::LineIndex;
use log::{as_debug, debug};
use lsp_types::{ConfigurationItem, Diagnostic, Url};
use mabo_parser::Schema;
use ouroboros::self_referencing;
use ropey::Rope;
use rustc_hash::FxHashMap;

use crate::{client::Client, config};

#[allow(clippy::module_name_repetitions)]
pub struct GlobalState<'a> {
    pub client: Client<'a>,
    pub files: FxHashMap<Url, File>,
    pub settings: config::Global,
}

#[self_referencing(pub_extras)]
pub struct File {
    rope: Rope,
    pub index: LineIndex,
    pub content: String,
    #[borrows(index, content)]
    #[covariant]
    pub schema: Result<Schema<'this>, Diagnostic>,
    #[borrows(schema)]
    #[covariant]
    pub simplified: Result<mabo_compiler::simplify::Schema<'this>, &'this Diagnostic>,
}

impl GlobalState<'_> {
    pub fn reload_settings(&mut self) -> Result<()> {
        let mut settings = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("mabo".to_owned()),
            }])
            .context("failed getting configuration from client")?;

        ensure!(
            settings.len() == 1,
            "configuration contains not exactly one object"
        );

        let settings = serde_json::from_value(settings.remove(0))
            .context("failed to parse raw configuration")?;

        debug!(settings = as_debug!(settings); "configuration loaded");

        self.settings = settings;

        Ok(())
    }
}
