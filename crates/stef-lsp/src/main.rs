#![allow(missing_docs)]

use std::collections::HashMap;

use directories::ProjectDirs;
use ouroboros::self_referencing;
use stef_parser::Schema;
use tokio::sync::Mutex;
use tower_lsp::{
    async_trait,
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams,
        InitializeResult, InitializedParams, MessageType, ServerCapabilities, ServerInfo,
        TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer, LspService, Server,
};
use tracing::{debug, Level};
use tracing_subscriber::{filter::Targets, fmt::MakeWriter, prelude::*};

use self::cli::Cli;

mod cli;
mod compile;
mod utf16;

#[derive(Debug)]
struct Backend {
    client: Client,
    files: Mutex<HashMap<Url, File>>,
}

#[self_referencing]
#[derive(Debug)]
struct File {
    content: String,
    #[borrows(content)]
    #[covariant]
    schema: std::result::Result<Schema<'this>, Diagnostic>,
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_CRATE_NAME").to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        debug!("initialized");
    }

    async fn shutdown(&self) -> Result<()> {
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let dirs = ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME")).unwrap();

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
            .expect("failed to open provided pipe/socket");
        let (read, write) = tokio::io::split(file);
        Server::new(read, write, socket).serve(service).await;
    } else if let Some(port) = cli.socket {
        unimplemented!("open TCP connection on port {port}");
    }
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
