#![warn(clippy::expect_used, clippy::unwrap_used)]
#![allow(missing_docs)]

use std::collections::HashMap;

use anyhow::{ensure, Context, Result};
use directories::ProjectDirs;
use ouroboros::self_referencing;
use stef_parser::Schema;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::{
    async_trait,
    jsonrpc::Result as LspResult,
    lsp_types::{
        ConfigurationItem, Diagnostic, DidChangeConfigurationParams, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
        InitializedParams, MessageType, Registration, SemanticTokenModifier, SemanticTokenType,
        SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
        SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
        WorkDoneProgressOptions,
    },
    Client, LanguageServer, LspService, Server,
};
use tracing::{debug, error, Level};
use tracing_subscriber::{filter::Targets, fmt::MakeWriter, prelude::*};

use self::cli::Cli;

mod cli;
mod compile;
mod config;
mod utf16;

#[derive(Debug)]
struct Backend {
    client: Client,
    files: Mutex<HashMap<Url, File>>,
    settings: RwLock<config::Global>,
}

#[self_referencing]
#[derive(Debug)]
struct File {
    content: String,
    #[borrows(content)]
    #[covariant]
    schema: Result<Schema<'this>, Diagnostic>,
}

impl Backend {
    async fn reload_settings(&self) -> Result<()> {
        let mut settings = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("stef".to_owned()),
            }])
            .await
            .context("failed getting configuration from client")?;

        ensure!(
            settings.len() == 1,
            "configuration contains not exactly one object"
        );

        let settings = serde_json::from_value(settings.remove(0))
            .context("failed to parse raw configuration")?;

        debug!("configuration loaded: {settings:#?}");

        *self.settings.write().await = settings;

        Ok(())
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_CRATE_NAME").to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: Some(false),
                            },
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::INTERFACE,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::TYPE_PARAMETER,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::EVENT,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::MACRO,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::MODIFIER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::REGEXP,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::DECORATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::ABSTRACT,
                                    SemanticTokenModifier::ASYNC,
                                    SemanticTokenModifier::MODIFICATION,
                                    SemanticTokenModifier::DOCUMENTATION,
                                    SemanticTokenModifier::DEFAULT_LIBRARY,
                                ],
                            },
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        if let Err(e) = self.reload_settings().await {
            error!(error = ?e, "failed loading initial settings");
        }

        if let Err(e) = self
            .client
            .register_capability(vec![Registration {
                id: "1".to_owned(),
                method: "workspace/didChangeConfiguration".to_owned(),
                register_options: None,
            }])
            .await
        {
            error!(error = ?e, "failed registering for configuration changes");
        }

        debug!("initialized");
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!(uri = %params.text_document.uri, "schema opened");

        let file = FileBuilder {
            content: params.text_document.text,
            schema_builder: |schema| compile::compile(schema),
        }
        .build();

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                file.borrow_schema()
                    .as_ref()
                    .err()
                    .map(|diag| vec![diag.clone()])
                    .unwrap_or_default(),
                None,
            )
            .await;

        self.files
            .lock()
            .await
            .insert(params.text_document.uri, file);
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        debug!(uri = %params.text_document.uri, "schema changed");

        let file = FileBuilder {
            content: params.content_changes.remove(0).text,
            schema_builder: |schema| compile::compile(schema),
        }
        .build();

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                file.borrow_schema()
                    .as_ref()
                    .err()
                    .map(|diag| vec![diag.clone()])
                    .unwrap_or_default(),
                None,
            )
            .await;

        self.files
            .lock()
            .await
            .insert(params.text_document.uri, file);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        debug!(uri = %params.text_document.uri, "schema closed");
        self.files.lock().await.remove(&params.text_document.uri);
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        debug!(uri = %params.text_document.uri, "requested semantic tokens");
        Ok(None)
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed");

        if let Err(e) = self.reload_settings().await {
            error!(error = ?e, "failed loading changed settings");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
        .context("failed locating project directories")?;

    let file_appender = tracing_appender::rolling::daily(dirs.cache_dir(), "log");
    let (file_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let (service, socket) = LspService::new(|client| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(ClientLogWriter::new(client.clone())),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
            .with(Targets::new().with_default(Level::WARN).with_targets([
                (env!("CARGO_CRATE_NAME"), Level::TRACE),
                ("stef_compiler", Level::TRACE),
                ("stef_parser", Level::TRACE),
                ("tower_lsp", Level::DEBUG),
            ]))
            .init();

        Backend {
            client,
            files: Mutex::default(),
            settings: RwLock::default(),
        }
    });

    if cli.stdio {
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        Server::new(stdin, stdout, socket).serve(service).await;
    } else if let Some(file) = cli.pipe {
        let file = tokio::fs::File::options()
            .read(true)
            .write(true)
            .open(file)
            .await
            .context("failed to open provided pipe/socket")?;

        let (read, write) = tokio::io::split(file);
        Server::new(read, write, socket).serve(service).await;
    } else if let Some(port) = cli.socket {
        unimplemented!("open TCP connection on port {port}");
    }

    Ok(())
}

struct ClientLogWriter {
    client: Client,
}

impl ClientLogWriter {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl std::io::Write for ClientLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let client = self.client.clone();
        let message = String::from_utf8_lossy(buf).trim().to_owned();

        tokio::spawn(async move { client.log_message(MessageType::LOG, message).await });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl MakeWriter<'_> for ClientLogWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        Self {
            client: self.client.clone(),
        }
    }
}
