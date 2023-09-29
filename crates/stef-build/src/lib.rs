#![deny(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::path::{Path, PathBuf};

use stef_parser::Schema;
use thiserror::Error;

pub use self::definition::compile_schema;

mod decode;
mod definition;
mod encode;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed reading schema file at {file:?}")]
    Read {
        #[source]
        source: std::io::Error,
        file: PathBuf,
    },
    #[error("failed parsing schema from {file:?}: {message}")]
    Parse { message: String, file: PathBuf },
}

pub fn compile(schemas: &[impl AsRef<Path>], _includes: &[impl AsRef<Path>]) -> Result<()> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    for schema in schemas {
        let path = schema.as_ref();

        let input = std::fs::read_to_string(path).map_err(|e| Error::Read {
            source: e,
            file: path.to_owned(),
        })?;

        let schema = Schema::parse(&input).map_err(|e| Error::Parse {
            message: e.to_string(),
            file: path.to_owned(),
        })?;
        let code = definition::compile_schema(&schema);
        let code = prettyplease::unparse(&syn::parse2(code).unwrap());

        println!("{code}");

        let out_file = out_dir.join(format!(
            "{}.rs",
            path.file_stem().unwrap().to_str().unwrap()
        ));

        std::fs::write(out_file, code).unwrap();
    }

    Ok(())
}
