//! Main command line interface for tooling support of Stef schema files.

use std::{fs, process::ExitCode};

use miette::{Context, IntoDiagnostic, Result};
use stef_parser::Schema;

use self::cli::Cli;

mod cli;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(cmd) = cli.cmd {
        let result = match cmd {
            cli::Command::Init { path } => {
                println!("TODO: create basic setup at {path:?}");
                Ok(())
            }
            cli::Command::Check { files } => check(files),
            cli::Command::Format { files } => format(files),
        };

        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:?}");
                ExitCode::FAILURE
            }
        };
    }

    ExitCode::SUCCESS
}

fn check(patterns: Vec<String>) -> Result<()> {
    for pattern in patterns {
        for entry in glob::glob(&pattern)
            .into_diagnostic()
            .wrap_err("Failed parsing glob pattern")?
        {
            let entry = entry.into_diagnostic().wrap_err("Failed reading entry")?;
            let buf = fs::read_to_string(&entry)
                .into_diagnostic()
                .wrap_err_with(|| format!("Failed reading {entry:?}"))?;

            if let Err(e) = Schema::parse(&buf, Some(&entry)).wrap_err("Failed parsing schema file")
            {
                eprintln!("{e:?}");
            }
        }
    }

    Ok(())
}

fn format(patterns: Vec<String>) -> Result<()> {
    for pattern in patterns {
        for entry in glob::glob(&pattern)
            .into_diagnostic()
            .wrap_err("Failed parsing glob pattern")?
        {
            let entry = entry.into_diagnostic().wrap_err("Failed reading entry")?;
            let buf = fs::read_to_string(&entry).into_diagnostic()?;
            let schema =
                match Schema::parse(&buf, Some(&entry)).wrap_err("Failed parsing schema file") {
                    Ok(schema) => schema,
                    Err(e) => {
                        eprintln!("{e:?}");
                        continue;
                    }
                };

            let formatted = schema.to_string();

            if buf != formatted {
                fs::write(entry, &formatted).into_diagnostic()?;
            }
        }
    }

    Ok(())
}
