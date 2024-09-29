//! Helper that renders all the CLI sub-commands and arguments into Markdown files for the book.
//!
//! Should be run whenever there are changes to the `src/cli.rs` file to reflect the changes in the
//! book accordingly.

use std::{fmt::Write, fs, path::Path};

use anyhow::{Context, Result};
use clap::CommandFactory;

use self::cli::Cli;

#[expect(dead_code)]
#[path = "../src/cli.rs"]
mod cli;

fn main() -> Result<()> {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../book/src/reference/cli/");
    let base = Path::new(base).canonicalize()?;

    let cmd = Cli::command();
    for sub in cmd.get_subcommands() {
        let mut buf = "---\n".to_owned();
        writeln!(&mut buf, "editLink: false")?;
        writeln!(&mut buf, "lastUpdated: false")?;
        writeln!(&mut buf, "---")?;

        writeln!(&mut buf, "\n# {} {}", cmd.get_name(), sub.get_name())?;

        for (i, alias) in sub.get_visible_aliases().enumerate() {
            if i == 0 {
                write!(&mut buf, "\n- Aliases: ")?;
            } else {
                write!(&mut buf, ", ")?;
            }
            write!(&mut buf, "`{alias}`")?;
        }

        if sub.get_visible_aliases().next().is_some() {
            writeln!(&mut buf)?;
        }

        if let Some(about) = sub.get_long_about() {
            writeln!(&mut buf, "\n{about}")?;
        } else if let Some(about) = sub.get_about() {
            writeln!(&mut buf, "\n{about}.")?;
        }

        writeln!(&mut buf, "\n## Arguments")?;
        for arg in sub.get_positionals() {
            writeln!(
                &mut buf,
                "\n### `{}`",
                arg.get_value_names()
                    .context("no value name for positional arg")?[0]
            )?;

            if let Some(help) = arg.get_long_help() {
                writeln!(&mut buf, "\n{help}")?;
            } else if let Some(help) = arg.get_help() {
                writeln!(&mut buf, "\n{help}.")?;
            }
        }

        writeln!(&mut buf, "\n## Options")?;
        for arg in sub.get_opts() {
            match (arg.get_long(), arg.get_short()) {
                (Some(long), Some(short)) => writeln!(&mut buf, "\n### `-{short}`, `--{long}`"),
                (Some(long), None) => writeln!(&mut buf, "\n### `--{long}`"),
                (None, Some(short)) => writeln!(&mut buf, "\n### `-{short}`"),
                (None, None) => continue,
            }?;

            if let Some(help) = arg.get_long_help() {
                writeln!(&mut buf, "\n{help}")?;
            } else if let Some(help) = arg.get_help() {
                writeln!(&mut buf, "\n{help}.")?;
            }
        }

        fs::write(base.join(format!("{}.md", sub.get_name())), buf)?;
    }

    Ok(())
}
