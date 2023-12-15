#![warn(clippy::expect_used, clippy::unwrap_used)]
#![allow(missing_docs)]

use std::{collections::HashMap, net::Ipv4Addr, time::Instant};

use anyhow::{bail, Result};
use log::{as_debug, debug, error, info, warn};
use lsp_server::{Connection, ErrorCode, ExtractError, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Initialized, Notification as LspNotification,
    },
    request::{
        DocumentSymbolRequest, HoverRequest, Request as LspRequest, SemanticTokensFullRequest,
        Shutdown,
    },
    DocumentSymbol, InitializeParams, SemanticTokens,
};

use self::{cli::Cli, client::Client};
use crate::state::GlobalState;

mod cli;
mod client;
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
                    HoverRequest::METHOD => {
                        handle_request::<HoverRequest, _>(
                            conn,
                            &mut state,
                            req,
                            handlers::hover,
                            |value| value,
                        )?;
                    }
                    DocumentSymbolRequest::METHOD => {
                        handle_request::<DocumentSymbolRequest, _>(
                            conn,
                            &mut state,
                            req,
                            handlers::document_symbol,
                            |value| value.unwrap_or(Vec::<DocumentSymbol>::default().into()),
                        )?;
                    }
                    SemanticTokensFullRequest::METHOD => {
                        handle_request::<SemanticTokensFullRequest, _>(
                            conn,
                            &mut state,
                            req,
                            handlers::semantic_tokens_full,
                            |value| value.unwrap_or(SemanticTokens::default().into()),
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
                    handle_notify::<Initialized>(&mut state, notif, handlers::initialized);
                }
                DidOpenTextDocument::METHOD => {
                    handle_notify::<DidOpenTextDocument>(&mut state, notif, handlers::did_open);
                }
                DidChangeTextDocument::METHOD => {
                    handle_notify::<DidChangeTextDocument>(&mut state, notif, handlers::did_change);
                }
                DidCloseTextDocument::METHOD => {
                    handle_notify::<DidCloseTextDocument>(&mut state, notif, handlers::did_close);
                }
                DidChangeConfiguration::METHOD => {
                    handle_notify::<DidChangeConfiguration>(
                        &mut state,
                        notif,
                        handlers::did_change_configuration,
                    );
                }
                _ => debug!(notification = as_debug!(notif); "got unknown notification"),
            },
        }
    }

    Ok(())
}

fn handle_request<T, R>(
    conn: &Connection,
    state: &mut GlobalState<'_>,
    req: Request,
    handler: fn(&mut GlobalState<'_>, T::Params) -> Result<T::Result>,
    post_process: fn(T::Result) -> R,
) -> Result<()>
where
    T: LspRequest,
    R: serde::Serialize,
{
    let (id, params) = match cast_req::<T>(req) {
        Ok(req) => req,
        Err((e, id)) => {
            return conn
                .sender
                .send(Response::new_err(id, ErrorCode::InvalidParams as _, e.to_string()).into())
                .map_err(Into::into)
        }
    };

    let start = Instant::now();
    let result = handler(state, params);

    debug!(duration = as_debug!(start.elapsed()); "handled request");

    conn.sender
        .send(
            match result {
                Ok(value) => Response::new_ok(id, post_process(value)),
                Err(e) => Response::new_err(id, ErrorCode::InternalError as _, e.to_string()),
            }
            .into(),
        )
        .map_err(Into::into)
}

fn cast_req<R>(req: Request) -> Result<(RequestId, R::Params), (ExtractError<Request>, RequestId)>
where
    R: LspRequest,
    R::Params: serde::de::DeserializeOwned,
{
    match serde_json::from_value(req.params) {
        Ok(params) => Ok((req.id, params)),
        Err(error) => Err((
            ExtractError::JsonError {
                method: req.method,
                error,
            },
            req.id,
        )),
    }
}

fn handle_notify<T>(
    state: &mut GlobalState<'_>,
    notif: Notification,
    handler: fn(&mut GlobalState<'_>, T::Params),
) where
    T: LspNotification,
{
    let params = match cast_notify::<T>(notif) {
        Ok(notif) => notif,
        Err(e) => {
            error!(error = as_debug!(e); "invalid notification parameters");
            return;
        }
    };

    handler(state, params);
}

fn cast_notify<R>(notif: Notification) -> Result<R::Params>
where
    R: LspNotification,
    R::Params: serde::de::DeserializeOwned,
{
    notif.extract(R::METHOD).map_err(Into::into)
}
