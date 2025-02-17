//! TODO

use std::{env, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use mabo_go::{Opts, Output};
use mabo_parser::Schema;

use self::cli::Cli;

mod cli;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let project_dir = match cli.project_dir {
        Some(dir) => dir,
        None => env::current_dir().context("failed locating project directory")?,
    };
    let out_dir = match cli.out_dir {
        Some(dir) => dir,
        None => env::current_dir().context("failed locating output directory")?,
    };

    ensure!(
        project_dir.join("Mabo.toml").exists(),
        "directory at {project_dir:?} doesn't appear to be a Mabo project"
    );

    let project = mabo_project::load(project_dir).context("failed loading project")?;

    fs::create_dir_all(&out_dir).context("failed creating output directory")?;

    let inputs = project
        .files
        .into_iter()
        .map(|path| {
            let input = fs::read_to_string(&path)
                .with_context(|| format!("failed reading schema file at {path:?}"))?;
            Ok((path, input))
        })
        .collect::<Result<Vec<_>>>()?;

    let validated = inputs
        .iter()
        .map(|(path, input)| {
            let stem = path
                .file_stem()
                .context("missing file name")?
                .to_str()
                .context("invalid utf-8 encoding")?;

            let schema = Schema::parse(input, Some(path))?;
            mabo_compiler::validate_schema(&schema)?;

            Ok((stem, schema))
        })
        .collect::<Result<Vec<_>>>()?;

    let validated = validated
        .iter()
        .map(|(name, schema)| (*name, schema))
        .collect::<Vec<_>>();

    mabo_compiler::resolve_schemas(&validated)?;

    let opts = Opts {
        package: &project.project_file.package.name,
    };

    for (_, schema) in validated {
        let schema = mabo_compiler::simplify_schema(schema);
        let code = mabo_go::render_schema(&opts, &schema);

        write_output(code, &out_dir)?;
    }

    if !cli.no_fmt {
        let output = Command::new("gofmt")
            .args(["-s", "-w"])
            .arg(out_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("failed to format output:\n{stderr}");
        }
    }

    Ok(())
}

fn write_output(output: Output<'_>, parent: &Path) -> Result<()> {
    let path = parent.join(output.name);

    fs::create_dir_all(&path)?;
    fs::write(path.join(format!("{}.go", output.name)), output.content)?;

    for module in output.modules {
        write_output(module, &path)?;
    }

    Ok(())
}
