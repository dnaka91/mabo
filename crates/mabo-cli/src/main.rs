//! Main command line interface for tooling support of Mabo schema files.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::Context;
use mabo_parser::Schema;
use miette::Context as _;

use self::cli::{CheckArgs, Cli, DocArgs, FmtArgs};

mod cli;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(cmd) = cli.cmd {
        let result = match cmd {
            cli::Command::Init(args) => {
                println!("TODO: create basic setup at {:?}", args.path);
                Ok(())
            }
            cli::Command::Check(args) => check(args),
            cli::Command::Fmt(args) => format(args),
            cli::Command::Doc(args) => doc(args),
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

fn check(args: CheckArgs) -> anyhow::Result<()> {
    for file in project_or_files(args.project_dir, args.files)? {
        let buf = fs::read_to_string(&file).with_context(|| format!("failed reading {file:?}"))?;

        if let Err(e) = Schema::parse(&buf, Some(&file)).wrap_err("failed parsing schema file") {
            eprintln!("{e:?}");
        }
    }

    Ok(())
}

fn format(args: FmtArgs) -> anyhow::Result<()> {
    for file in project_or_files(args.project_dir, args.files)? {
        let buf = fs::read_to_string(&file)?;
        let schema = match Schema::parse(&buf, Some(&file)).wrap_err("Failed parsing schema file") {
            Ok(schema) => schema,
            Err(e) => {
                eprintln!("{e:?}");
                continue;
            }
        };

        let formatted = schema.to_string();

        if buf != formatted {
            fs::write(file, &formatted)?;
        }
    }

    Ok(())
}

fn doc(args: DocArgs) -> anyhow::Result<()> {
    let project = mabo_project::load(project_dir(args.project_dir)?)?;

    for file in project.files {
        let content = std::fs::read_to_string(&file)?;
        let schema = mabo_parser::Schema::parse(&content, Some(&file))?;
        let schema = mabo_compiler::simplify_schema(&schema);
        let docs = mabo_doc::render_schema(&mabo_doc::Opts {}, &schema)?;

        write_doc_output(&docs, &args.out_dir.join(docs.name))?;
    }

    Ok(())
}

fn write_doc_output(output: &mabo_doc::Output<'_>, parent: &Path) -> anyhow::Result<()> {
    let path = parent.join(output.name);
    let file = parent.join(&output.file);

    fs::create_dir_all(file.parent().unwrap())?;
    fs::write(file, &output.content)?;

    for module in &output.modules {
        write_doc_output(module, &path)?;
    }

    Ok(())
}

fn project_or_files(
    project: Option<PathBuf>,
    patterns: Vec<String>,
) -> anyhow::Result<Vec<PathBuf>> {
    Ok(if patterns.is_empty() {
        mabo_project::load(project_dir(project)?)?.files
    } else {
        let mut files = Vec::new();
        for pattern in patterns {
            for entry in glob::glob(&pattern).context("failed parsing glob pattern")? {
                let entry = entry.context("failed reading entry")?;
                if !entry.extension().map_or(false, |ext| ext == "mabo") {
                    continue;
                }
                files.push(entry);
            }
        }
        files
    })
}

fn project_dir(arg: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    arg.map_or_else(
        || std::env::current_dir().context("failed finding current directory"),
        Ok,
    )
}
