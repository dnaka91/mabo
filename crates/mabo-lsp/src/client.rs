use std::time::Duration;

use anyhow::{bail, ensure, Result};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    notification::{Notification as LspNotification, PublishDiagnostics},
    request::{RegisterCapability, Request as LspRequest, WorkspaceConfiguration},
    ConfigurationItem, ConfigurationParams, Diagnostic, PublishDiagnosticsParams, Registration,
    RegistrationParams, Uri,
};

pub struct Client<'a> {
    conn: &'a Connection,
    next_id: i32,
}

impl<'a> Client<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn, next_id: 0 }
    }

    fn next_id(&mut self) -> i32 {
        let id = self.next_id.wrapping_add(1);
        self.next_id = id;
        id
    }

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
        let req_id = self.next_id();

        self.conn.sender.send_timeout(
            Request::new(req_id.into(), T::METHOD.to_owned(), params).into(),
            Duration::from_secs(2),
        )?;

        match self.conn.receiver.recv_timeout(Duration::from_secs(2))? {
            Message::Response(Response {
                id,
                result: Some(result),
                error: None,
            }) => {
                ensure!(id == req_id.into(), "invalid ID");
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

    pub fn publish_diagnostics(
        &self,
        uri: Uri,
        diagnostics: Vec<Diagnostic>,
        version: Option<i32>,
    ) -> Result<()> {
        self.send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        })
    }

    pub fn configuration(
        &mut self,
        items: Vec<ConfigurationItem>,
    ) -> Result<Vec<serde_json::Value>> {
        self.send_request::<WorkspaceConfiguration>(ConfigurationParams { items })
    }

    pub fn register_capability(&mut self, registrations: Vec<Registration>) -> Result<()> {
        self.send_request::<RegisterCapability>(RegistrationParams { registrations })
    }
}
