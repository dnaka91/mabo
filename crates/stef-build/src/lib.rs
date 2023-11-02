#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::{
    convert::AsRef,
    fmt::Debug,
    path::{Path, PathBuf},
};

use miette::Report;
use stef_parser::Schema;
use thiserror::Error;

pub use self::definition::compile_schema;

mod decode;
mod definition;
mod encode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error)]
pub enum Error {
    #[error("failed to parse the glob pattern {glob:?}")]
    Pattern {
        #[source]
        source: glob::PatternError,
        glob: String,
    },
    #[error("failed to read files of a glob pattern")]
    Glob {
        #[source]
        source: glob::GlobError,
    },
    #[error("failed reading schema file at {file:?}")]
    Read {
        #[source]
        source: std::io::Error,
        file: PathBuf,
    },
    #[error("failed parsing schema from {file:?}:\n{report:?}")]
    Parse { report: Report, file: PathBuf },
    #[error("failed compiling schema from {file:?}:\n{report:?}")]
    Compile { report: Report, file: PathBuf },
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

pub fn compile(schemas: &[impl AsRef<str>], _includes: &[impl AsRef<Path>]) -> Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .color(true)
                .context_lines(3)
                .force_graphical(true)
                .terminal_links(true)
                .build(),
        )
    }))
    .ok();

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut inputs = Vec::new();
    let mut validated = Vec::new();

    for schema in schemas.iter().map(AsRef::as_ref) {
        for schema in glob::glob(schema).map_err(|source| Error::Pattern {
            source,
            glob: schema.to_owned(),
        })? {
            let path = schema.map_err(|e| Error::Glob { source: e })?;

            let input = std::fs::read_to_string(&path).map_err(|source| Error::Read {
                source,
                file: path.clone(),
            })?;

            inputs.push((path, input));
        }
    }

    for (path, input) in &inputs {
        let stem = path.file_stem().unwrap().to_str().unwrap();

        let schema = Schema::parse(input, Some(path)).map_err(|e| Error::Parse {
            report: Report::new(e),
            file: path.clone(),
        })?;

        stef_compiler::validate_schema(&schema).map_err(|e| Error::Compile {
            report: Report::new(e),
            file: path.clone(),
        })?;

        validated.push((stem, schema));
    }

    let validated = validated
        .iter()
        .map(|(name, schema)| (*name, schema))
        .collect::<Vec<_>>();

    stef_compiler::resolve_schemas(&validated).map_err(|e| Error::Compile {
        report: Report::new(e),
        file: PathBuf::new(),
    })?;

    for (stem, schema) in validated {
        let code = definition::compile_schema(schema);
        let code = prettyplease::unparse(&syn::parse2(code).unwrap());

        let out_file = out_dir.join(format!("{stem}.rs",));

        std::fs::write(out_file, code).unwrap();
    }

    Ok(())
}
