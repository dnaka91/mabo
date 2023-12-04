#![allow(missing_docs)]

use std::collections::HashMap;

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
use tracing::debug;
use tracing_subscriber::EnvFilter;

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
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
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
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer({
                let client = client.clone();
                move || ClientLogWriter {
                    client: client.clone(),
                }
            })
            .init();
        Backend {
            client,
            files: Mutex::default(),
        }
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

struct ClientLogWriter {
    client: Client,
}

impl std::io::Write for ClientLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let client = self.client.clone();
        let message = String::from_utf8_lossy(buf).into_owned();

        tokio::spawn(async move { client.log_message(MessageType::INFO, message).await });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
