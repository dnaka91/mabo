use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about, author, version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(alias = "initialize")]
    Init(InitArgs),
    Check(CheckArgs),
    #[command(alias = "format")]
    Fmt(FmtArgs),
    #[command(alias = "document")]
    Doc(DocArgs),
}

/// Arguments for the [`Command::Init`] subcommand.
#[derive(Args)]
pub struct InitArgs {
    #[arg(long)]
    pub name: Option<String>,
    pub path: Option<PathBuf>,
}

/// Arguments for the [`Command::Check`] subcommand.
#[derive(Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    pub files: Vec<String>,
}

/// Arguments for the [`Command::Fmt`] subcommand.
#[derive(Args)]
pub struct FmtArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    pub files: Vec<String>,
}

/// Arguments for the [`Command::Doc`] subcommand.
#[derive(Args)]
pub struct DocArgs {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    pub out_dir: PathBuf,
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
