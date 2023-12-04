use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Parser)]
#[command(about, author, version)]
pub struct Cli {
    /// Use standard I/O as the communication channel.
    #[arg(long, conflicts_with_all = ["pipe", "socket"])]
    pub stdio: bool,
    /// Use pipes (Windows) or socket files (Linux, Mac) as the communication channel.
    #[arg(
        long,
        conflicts_with_all = ["socket"],
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
    )]
    pub pipe: Option<PathBuf>,
    /// Use a socket as the communication channel.
    #[arg(long, exclusive = true, value_name = "PORT")]
    pub socket: Option<u16>,
    /// Process ID of the editor that started this server.
    ///
    /// To support the case that the editor starting a server crashes an editor should also pass
    /// its process id to the server. This allows the server to monitor the editor process and to
    /// shutdown itself if the editor process dies.
    #[arg(long = "clientProcessId", value_name = "ID")]
    pub client_process_id: Option<i32>,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
