use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about, author, version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Init {
        path: Option<PathBuf>,
    },
    Check {
        #[arg(num_args(1..))]
        files: Vec<PathBuf>,
    },
    Format {
        #[arg(num_args(1..))]
        files: Vec<PathBuf>,
    },
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
