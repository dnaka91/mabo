use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, author, version, propagate_version = true)]
pub struct Cli {
    #[arg(num_args(1..))]
    files: Vec<String>,
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
