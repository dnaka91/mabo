#![warn(clippy::expect_used, clippy::unwrap_used)]
#![allow(missing_docs)]

use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::{bail, Result};
use log::{as_debug, debug, error, info, warn};
use lsp_server::{Connection, ErrorCode, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Initialized, Notification as LspNotification,
    },
    request::{Request as LspRequest, SemanticTokensFullRequest, Shutdown},
    InitializeParams, SemanticTokens, SemanticTokensResult,
};

use self::{cli::Cli, client::Client};
use crate::state::GlobalState;

mod cli;
mod client;
mod compile;
mod config;
mod handlers;
mod logging;
mod state;

fn main() -> Result<()> {
    let cli = Cli::parse();
    logging::init(None)?;

    let (connection, io_threads) = if cli.stdio {
        Connection::stdio()
    } else if let Some(file) = cli.pipe {
        unimplemented!("open connection on pipe/socket {file:?}");
    } else if let Some(port) = cli.socket {
        Connection::connect((Ipv4Addr::LOCALHOST, port))?
    } else {
        bail!("no connection method provided")
    };

    let mut state = GlobalState {
        client: Client::new(&connection),
        files: HashMap::default(),
        settings: config::Global::default(),
    };

    let (id, params) = connection.initialize_start()?;
    let init_params = serde_json::from_value::<InitializeParams>(params)?;
    let init_result = handlers::initialize(&mut state, init_params)?;
    connection.initialize_finish(id, serde_json::to_value(init_result)?)?;

    info!("server initialized");

    if let Err(e) = main_loop(&connection, state) {
        error!(error = as_debug!(e); "error in main loop");
        return Err(e);
    }

    drop(connection);
    io_threads.join()?;

    info!("goodbye!");

    Ok(())
}

fn main_loop(conn: &Connection, mut state: GlobalState<'_>) -> Result<()> {
    for msg in &conn.receiver {
        match msg {
            lsp_server::Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    info!("shutting down");
                    return Ok(());
                }

                match req.method.as_str() {
                    Shutdown::METHOD => {
                        warn!("should never reach this");
                    }
                    SemanticTokensFullRequest::METHOD => {
                        let (id, params) = cast_req::<SemanticTokensFullRequest>(req)?;
                        let result = handlers::semantic_tokens_full(&mut state, params);

                        conn.sender.send(
                            match result {
                                Ok(value) => Response::new_ok(
                                    id,
                                    value.unwrap_or(SemanticTokensResult::Tokens(
                                        SemanticTokens::default(),
                                    )),
                                ),
                                Err(e) => Response::new_err(
                                    id,
                                    ErrorCode::InternalError as _,
                                    e.to_string(),
                                ),
                            }
                            .into(),
                        )?;
                    }

                    _ => {
                        debug!(request = as_debug!(req); "got unsupported request");
                        conn.sender.send(
                            Response::new_err(
                                req.id,
                                ErrorCode::MethodNotFound as _,
                                format!("request `{}` not supported", req.method),
                            )
                            .into(),
                        )?;
                    }
                }
            }
            lsp_server::Message::Response(resp) => {
                debug!(response = as_debug!(resp); "got unexpected response");
            }
            lsp_server::Message::Notification(notif) => match notif.method.as_str() {
                Initialized::METHOD => {
                    handlers::initialized(&mut state, cast_notify::<Initialized>(notif)?);
                }
                DidOpenTextDocument::METHOD => {
                    handlers::did_open(&mut state, cast_notify::<DidOpenTextDocument>(notif)?);
                }
                DidChangeTextDocument::METHOD => {
                    handlers::did_change(&mut state, cast_notify::<DidChangeTextDocument>(notif)?);
                }
                DidCloseTextDocument::METHOD => {
                    handlers::did_close(&mut state, cast_notify::<DidCloseTextDocument>(notif)?);
                }
                DidChangeConfiguration::METHOD => {
                    handlers::did_change_configuration(
                        &mut state,
                        cast_notify::<DidChangeConfiguration>(notif)?,
                    );
                }
                _ => debug!(notification = as_debug!(notif); "got unknown notification"),
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
