use std::{fs::File, io::Write, path::PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{kv::Visitor, Level, LevelFilter, Metadata, Record};
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    notification::{LogMessage, Notification as _},
    LogMessageParams, MessageType,
};
use parking_lot::Mutex;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime, UtcOffset};

static FORMAT_HMS: &[FormatItem<'_>] = format_description!("[hour]:[minute]:[second]");

pub fn init(conn: Option<Connection>) -> Result<()> {
    let offset = UtcOffset::current_local_offset()?;

    let dirs = ProjectDirs::from("rocks", "dnaka91", env!("CARGO_PKG_NAME"))
        .context("failed locating project directories")?;

    log::set_max_level(LevelFilter::Trace);
    log::set_boxed_logger(Box::new(CombinedLogger {
        client: conn.map(|conn| ClientLogger::new(conn, offset)),
        file: FileLogger::new(dirs.cache_dir().join("lsp.log"), offset)?,
        stderr: StderrLogger::new(offset),
    }))
    .map_err(Into::into)
}

fn write_message<'a>(
    f: &mut impl std::io::Write,
    record: &'a Record<'a>,
    offset: UtcOffset,
) -> Result<()> {
    OffsetDateTime::now_utc()
        .to_offset(offset)
        .format_into(f, FORMAT_HMS)?;

    write!(
        f,
        " {:5} {}: {}",
        record.level(),
        record.target(),
        record.args()
    )?;

    record.key_values().visit(&mut FormatVisitor(f))?;

    writeln!(f).map_err(Into::into)
}

struct FormatVisitor<'a, T>(&'a mut T);

impl<T: std::io::Write> Visitor<'_> for FormatVisitor<'_, T> {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'_>,
        value: log::kv::Value<'_>,
    ) -> Result<(), log::kv::Error> {
        write!(self.0, " {}=", key.as_str())?;
        value.visit(self)
    }
}

impl<T: std::io::Write> log::kv::value::Visit<'_> for FormatVisitor<'_, T> {
    fn visit_any(&mut self, value: log::kv::Value<'_>) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_u64(&mut self, value: u64) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_i64(&mut self, value: i64) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_u128(&mut self, value: u128) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_i128(&mut self, value: i128) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_f64(&mut self, value: f64) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_bool(&mut self, value: bool) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_borrowed_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_char(&mut self, value: char) -> Result<(), log::kv::Error> {
        write!(self.0, "{value}").map_err(Into::into)
    }

    fn visit_error(
        &mut self,
        err: &(dyn std::error::Error + 'static),
    ) -> Result<(), log::kv::Error> {
        write!(self.0, "{err}").map_err(Into::into)
    }

    fn visit_borrowed_error(
        &mut self,
        err: &(dyn std::error::Error + 'static),
    ) -> Result<(), log::kv::Error> {
        write!(self.0, "{err}").map_err(Into::into)
    }
}

struct ClientLogger {
    conn: Connection,
    offset: UtcOffset,
}

impl ClientLogger {
    fn new(conn: Connection, offset: UtcOffset) -> Self {
        Self { conn, offset }
    }
}

impl log::Log for ClientLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let message = {
            let mut buf = Vec::new();
            if let Err(e) = write_message(&mut buf, record, self.offset) {
                eprintln!("failed formatting log message: {e:?}");
                return;
            }
            String::from_utf8_lossy(&buf).into_owned()
        };

        let params = match serde_json::to_value(LogMessageParams {
            typ: MessageType::LOG,
            message,
        }) {
            Ok(params) => params,
            Err(e) => {
                eprintln!("failed serializing log message params: {e:?}");
                return;
            }
        };

        if let Err(e) = self.conn.sender.send(Message::Notification(Notification {
            method: LogMessage::METHOD.to_owned(),
            params,
        })) {
            eprintln!("failed sending log message to client: {e:?}");
        }
    }

    fn flush(&self) {}
}

struct FileLogger {
    file: Mutex<File>,
    offset: UtcOffset,
}

impl FileLogger {
    fn new(file: PathBuf, offset: UtcOffset) -> Result<Self> {
        Ok(Self {
            file: File::options().create(true).append(true).open(file)?.into(),
            offset,
        })
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Err(e) = write_message(&mut *self.file.lock(), record, self.offset) {
            eprintln!("failed writing log message to file: {e:?}");
        }
    }

    fn flush(&self) {
        if let Err(e) = self.file.lock().flush() {
            eprintln!("failed flushing log file: {e:?}");
        }
    }
}

struct StderrLogger {
    offset: UtcOffset,
}

impl StderrLogger {
    fn new(offset: UtcOffset) -> Self {
        Self { offset }
    }
}

impl log::Log for StderrLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Err(e) = write_message(&mut std::io::stderr(), record, self.offset) {
            eprintln!("failed formatting log message: {e:?}");
        }
    }

    fn flush(&self) {
        std::io::stderr().flush().ok();
    }
}

struct CombinedLogger {
    client: Option<ClientLogger>,
    file: FileLogger,
    stderr: StderrLogger,
}

impl log::Log for CombinedLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Warn
            || (metadata.target().starts_with(env!("CARGO_CRATE_NAME"))
                && metadata.level() <= Level::Trace)
            || (metadata.target().starts_with("stef_compiler") && metadata.level() <= Level::Trace)
            || (metadata.target().starts_with("stef_parser") && metadata.level() <= Level::Trace)
            || (metadata.target().starts_with("lsp_server") && metadata.level() <= Level::Info)
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            if let Some(client) = &self.client {
                client.log(record);
            }
            self.file.log(record);
            self.stderr.log(record);
        }
    }

    fn flush(&self) {
        if let Some(client) = &self.client {
            client.flush();
        }
        self.file.flush();
        self.stderr.flush();
    }
}
