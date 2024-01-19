use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueHint};

/// Command line interface to manage and support Mabo schema projects.
#[derive(Parser)]
#[command(name = "mabo", about, author, version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new project.
    #[command(visible_aliases = ["i", "initialize"])]
    Init(InitArgs),
    /// Check that a project or set of files are valid schemas.
    ///
    /// This involves first checking each schema individually to be parseable, then run various
    /// lints over it and finally resolve any external schema types.  In case any of the steps fail
    /// an error will be returned.
    #[command(visible_aliases = ["c"])]
    Check(CheckArgs),
    /// Format a project or set of files.
    #[command(visible_aliases = ["f", "format"])]
    Fmt(FmtArgs),
    /// Generate documentation for a project.
    #[command(visible_aliases = ["d", "document"])]
    Doc(DocArgs),
}

/// Arguments for the [`Command::Init`] subcommand.
#[derive(Args)]
pub struct InitArgs {
    /// Name of the project. If omitted, the name is derived from the current working directory.
    ///
    /// This is used as the project name in the `Mabo.toml` file to give it a unique identifier. It
    /// must only be unique within the project (in case it has multiple projects).
    #[arg(long)]
    pub name: Option<String>,
    /// Alternative path were the project should be generated.
    ///
    /// By default, the current directory is assumed to be the to-be project directory. This is
    /// where the `Mabo.toml` file will be placed to mark it as project. Therefore, the path must
    /// point to a directory, not a file.
    #[arg(value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,
}

/// Arguments for the [`Command::Check`] subcommand.
#[derive(Args)]
pub struct CheckArgs {
    /// Alternative location of the project directory containing a `Mabo.toml` file.
    ///
    /// By default, the current directory is assumed to be the project directory. This is the root
    /// from where the command operates. Therefore, using it has the same effect as moving to the
    /// project directory and executing the command without it.
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub project_dir: Option<PathBuf>,
    /// Loose list of glob patterns for files that should be checked instead of a project.
    ///
    /// Using this will disable the loading of a project and instead locate the files from the glob
    /// patterns, then treat them as one single set. The files will be treated as a single project
    /// but the `Mabo.toml` file is fully ignored.
    #[arg(conflicts_with = "project_dir")]
    pub files: Vec<String>,
}

/// Arguments for the [`Command::Fmt`] subcommand.
#[derive(Args)]
pub struct FmtArgs {
    /// Alternative location of the project directory containing a `Mabo.toml` file.
    ///
    /// By default, the current directory is assumed to be the project directory. This is the root
    /// from where the command operates. Therefore, using it has the same effect as moving to the
    /// project directory and executing the command without it.
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub project_dir: Option<PathBuf>,
    /// Loose list of glob patterns for files that should be formatted instead of a project.
    ///
    /// Using this will disable the loading of a project and instead locate the files from the glob
    /// patterns, then treat them as one single set. The files will be treated as a single project
    /// but the `Mabo.toml` file is fully ignored.
    #[arg(conflicts_with = "project_dir")]
    pub files: Vec<String>,
}

/// Arguments for the [`Command::Doc`] subcommand.
#[derive(Args)]
pub struct DocArgs {
    /// Alternative location of the project directory containing a `Mabo.toml` file.
    ///
    /// By default, the current directory is assumed to be the project directory. This is the root
    /// from where the command operates. Therefore, using it has the same effect as moving to the
    /// project directory and executing the command without it.
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub project_dir: Option<PathBuf>,
    /// Directory where the documentation files are written to.
    ///
    /// Note that this directory will be overwritten without confirmation. Any existing files will
    /// be replaced, but the directory is not fully cleared beforehand. That means any unrelated
    /// files not having the same name as any documentation file, will remain.
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
