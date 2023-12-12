#![warn(clippy::expect_used, clippy::unwrap_used)]
#![allow(missing_docs)]

use std::{collections::HashMap, net::Ipv4Addr, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use log::{as_debug, as_display, debug, error, info};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Initialized, Notification as LspNotification, PublishDiagnostics,
    },
    request::{
        RegisterCapability, Request as LspRequest, SemanticTokensFullRequest, Shutdown,
        WorkspaceConfiguration,
    },
    ConfigurationItem, ConfigurationParams, Diagnostic, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, PositionEncodingKind,
    PublishDiagnosticsParams, Registration, RegistrationParams, SemanticTokenModifier,
    SemanticTokenType, SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkDoneProgressOptions,
};
use ouroboros::self_referencing;
use stef_parser::Schema;

use self::cli::Cli;

mod cli;
mod compile;
mod config;
mod logging;

struct Backend {
    conn: Connection,
    files: HashMap<Url, File>,
    settings: config::Global,
    next_id: i32,
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
    fn send_notification<T>(&self, params: T::Params) -> Result<()>
    where
        T: LspNotification,
    {
        self.conn
            .sender
            .send_timeout(
                Notification::new(T::METHOD.to_owned(), params).into(),
                Duration::from_secs(2),
            )
            .map_err(Into::into)
    }

    fn send_request<T>(&mut self, params: T::Params) -> Result<T::Result>
    where
        T: LspRequest,
    {
        let next_id = self.next_id.wrapping_add(1);
        self.next_id = next_id;

        self.conn.sender.send_timeout(
            Request::new(next_id.into(), T::METHOD.to_owned(), params).into(),
            Duration::from_secs(2),
        )?;

        match self.conn.receiver.recv_timeout(Duration::from_secs(2))? {
            Message::Response(Response {
                id,
                result: Some(result),
                error: None,
            }) => {
                ensure!(id == next_id.into(), "invalid ID");
                serde_json::from_value(result).map_err(Into::into)
            }
            Message::Response(Response {
                id,
                result: None,
                error: Some(error),
            }) => bail!("request {id} failed: {error:?}"),
            _ => bail!("invalid message type"),
        }
    }

    fn publish_diagnostics(
        &self,
        uri: Url,
        diagnostics: Vec<Diagnostic>,
        version: Option<i32>,
    ) -> Result<()> {
        self.send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        })
    }

    fn configuration(&mut self, items: Vec<ConfigurationItem>) -> Result<Vec<serde_json::Value>> {
        self.send_request::<WorkspaceConfiguration>(ConfigurationParams { items })
    }

    fn register_capability(&mut self, registrations: Vec<Registration>) -> Result<()> {
        self.send_request::<RegisterCapability>(RegistrationParams { registrations })
    }

    fn reload_settings(&mut self) -> Result<()> {
        let mut settings = self
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("stef".to_owned()),
            }])
            .context("failed getting configuration from client")?;

        ensure!(
            settings.len() == 1,
            "configuration contains not exactly one object"
        );

        let settings = serde_json::from_value(settings.remove(0))
            .context("failed to parse raw configuration")?;

        debug!("configuration loaded: {settings:#?}");

        self.settings = settings;

        Ok(())
    }
}

trait LanguageServer {
    fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult>;
    fn initialized(&mut self, params: InitializedParams);
    fn shutdown(&mut self) -> Result<()>;
    fn did_open(&mut self, params: DidOpenTextDocumentParams);
    fn did_change(&mut self, params: DidChangeTextDocumentParams);
    fn did_close(&mut self, params: DidCloseTextDocumentParams);
    fn semantic_tokens_full(
        &mut self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>>;
    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams);
}

impl LanguageServer for Backend {
    fn initialize(&mut self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_CRATE_NAME").to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
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

    fn initialized(&mut self, _params: InitializedParams) {
        if let Err(e) = self.reload_settings() {
            error!(error = as_debug!(e); "failed loading initial settings");
        }

        if let Err(e) = self.register_capability(vec![Registration {
            id: "1".to_owned(),
            method: "workspace/didChangeConfiguration".to_owned(),
            register_options: None,
        }]) {
            error!(error = as_debug!(e); "failed registering for configuration changes");
        }

        debug!("initialized");
    }

    fn shutdown(&mut self) -> Result<()> {
        debug!("got shutdown request");
        Ok(())
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        debug!(uri = as_display!(params.text_document.uri); "schema opened");

        let file = FileBuilder {
            content: params.text_document.text,
            schema_builder: |schema| compile::compile(params.text_document.uri.clone(), schema),
        }
        .build();

        if let Err(e) = self.publish_diagnostics(
            params.text_document.uri.clone(),
            file.borrow_schema()
                .as_ref()
                .err()
                .map(|diag| vec![diag.clone()])
                .unwrap_or_default(),
            None,
        ) {
            error!(error = as_debug!(e); "failed publishing diagnostics");
        }

        self.files.insert(params.text_document.uri, file);
    }

    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) {
        debug!(uri = as_display!(params.text_document.uri); "schema changed");

        let file = FileBuilder {
            content: params.content_changes.remove(0).text,
            schema_builder: |schema| compile::compile(params.text_document.uri.clone(), schema),
        }
        .build();

        if let Err(e) = self.publish_diagnostics(
            params.text_document.uri.clone(),
            file.borrow_schema()
                .as_ref()
                .err()
                .map(|diag| vec![diag.clone()])
                .unwrap_or_default(),
            None,
        ) {
            error!(error = as_debug!(e); "failed publishing diagnostics");
        }

        self.files.insert(params.text_document.uri, file);
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        debug!(uri = as_display!(params.text_document.uri); "schema closed");
        self.files.remove(&params.text_document.uri);
    }

    fn semantic_tokens_full(
        &mut self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        debug!(uri = as_display!(params.text_document.uri); "requested semantic tokens");
        Ok(None)
    }

    fn did_change_configuration(&mut self, _params: DidChangeConfigurationParams) {
        debug!("configuration changed");

        if let Err(e) = self.reload_settings() {
            error!(error = as_debug!(e); "failed loading changed settings");
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    logging::init(None)?;

    let (connection, _io_threads) = if cli.stdio {
        Connection::stdio()
    } else if let Some(file) = cli.pipe {
        unimplemented!("open connection on pipe/socket {file:?}");
    } else if let Some(port) = cli.socket {
        Connection::connect((Ipv4Addr::LOCALHOST, port))?
    } else {
        bail!("no connection method provided")
    };

    let mut server = Backend {
        conn: Connection {
            sender: connection.sender.clone(),
            receiver: connection.receiver.clone(),
        },
        files: HashMap::default(),
        settings: config::Global::default(),
        next_id: 0,
    };

    let (id, params) = connection.initialize_start()?;
    let init_params = serde_json::from_value::<InitializeParams>(params)?;
    let init_result = server.initialize(init_params)?;
    connection.initialize_finish(id, serde_json::to_value(init_result)?)?;

    info!("server initialized");

    if let Err(e) = main_loop(&connection, server) {
        error!(error = as_debug!(e); "error in main loop");
        return Err(e);
    }

    // TODO: investigate why this hangs
    // io_threads.join()?;

    info!("goodbye!");

    Ok(())
}

fn main_loop(conn: &Connection, mut server: impl LanguageServer) -> Result<()> {
    for msg in &conn.receiver {
        match msg {
            lsp_server::Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    info!("shutting down");
                    return Ok(());
                }

                match req.method.as_str() {
                    Shutdown::METHOD => {
                        server.shutdown()?;
                    }
                    SemanticTokensFullRequest::METHOD => {
                        let (id, params) = cast_req::<SemanticTokensFullRequest>(req)?;
                        let result = server.semantic_tokens_full(params)?;

                        conn.sender.send(
                            Response::new_ok(
                                id,
                                result.unwrap_or(SemanticTokensResult::Tokens(
                                    SemanticTokens::default(),
                                )),
                            )
                            .into(),
                        )?;
                    }

                    _ => debug!("got request: {req:?}"),
                }
            }
            lsp_server::Message::Response(resp) => {
                debug!("got response: {resp:?}");
            }
            lsp_server::Message::Notification(notif) => match notif.method.as_str() {
                Initialized::METHOD => {
                    server.initialized(cast_notify::<Initialized>(notif)?);
                }
                DidOpenTextDocument::METHOD => {
                    server.did_open(cast_notify::<DidOpenTextDocument>(notif)?);
                }
                DidChangeTextDocument::METHOD => {
                    server.did_change(cast_notify::<DidChangeTextDocument>(notif)?);
                }
                DidCloseTextDocument::METHOD => {
                    server.did_close(cast_notify::<DidCloseTextDocument>(notif)?);
                }
                DidChangeConfiguration::METHOD => {
                    server.did_change_configuration(cast_notify::<DidChangeConfiguration>(notif)?);
                }
                _ => debug!("got unknown notification: {notif:?}"),
            },
        }
    }

    Ok(())
}

fn cast_req<R>(req: Request) -> Result<(RequestId, R::Params)>
where
    R: LspRequest,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD).map_err(Into::into)
}

fn cast_notify<R>(notif: Notification) -> Result<R::Params>
where
    R: LspNotification,
    R::Params: serde::de::DeserializeOwned,
{
    notif.extract(R::METHOD).map_err(Into::into)
}
