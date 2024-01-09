use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, author, version, propagate_version = true)]
pub struct Cli {
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
    #[arg(long, short)]
    pub out_dir: Option<PathBuf>,
    #[arg(long)]
    pub no_fmt: bool,
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
