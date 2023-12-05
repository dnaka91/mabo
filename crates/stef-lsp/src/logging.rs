use std::io::Write;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use time::{format_description::FormatItem, macros::format_description, UtcOffset};
use tower_lsp::{lsp_types::MessageType, Client};
use tracing::Level;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::{
    filter::Targets,
    fmt::{time::OffsetTime, MakeWriter},
    prelude::*,
};

pub fn prepare() -> Result<(Options, Guard)> {
    let timer = OffsetTime::new(
        UtcOffset::current_local_offset().context("failed retrieving local UTC offset")?,
        format_description!("[hour]:[minute]:[second]"),
    );

    let dirs = ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
        .context("failed locating project directories")?;

    let file_appender = tracing_appender::rolling::daily(dirs.cache_dir(), "log");
    let (file_appender, guard) = tracing_appender::non_blocking(file_appender);

    Ok((
        Options {
            timer,
            file_appender,
        },
        Guard(guard),
    ))
}

pub fn init(
    Options {
        timer,
        file_appender,
    }: Options,
    client: Client,
) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_timer(timer.clone())
                .with_writer(ClientLogWriter::new(client)),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(timer)
                .with_writer(file_appender),
        )
        .with(Targets::new().with_default(Level::WARN).with_targets([
            (env!("CARGO_CRATE_NAME"), Level::TRACE),
            ("stef_compiler", Level::TRACE),
            ("stef_parser", Level::TRACE),
            ("tower_lsp", Level::DEBUG),
        ]))
        .init();
}

pub struct Options {
    timer: OffsetTime<&'static [FormatItem<'static>]>,
    file_appender: NonBlocking,
}

pub struct Guard(tracing_appender::non_blocking::WorkerGuard);

struct ClientLogWriter {
    client: Client,
}

impl ClientLogWriter {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Write for ClientLogWriter {
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
