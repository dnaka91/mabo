//! Code generator crate for Rust projects that can be used in `build.rs` build scripts.

use std::{env, fmt::Debug, fs, path::PathBuf};

use mabo_parser::Schema;
use miette::Report;
use thiserror::Error;

pub use self::definition::compile_schema;

mod decode;
mod definition;
mod encode;
mod size;

/// Shorthand for the standard result type, that defaults to the crate level's [`Error`](enum@Error)
/// type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can happen when generating Rust source code from Mabo schema files.
#[derive(Error)]
pub enum Error {
    /// Failed to load the Mabo project.
    #[error("failed to load the Mabo project")]
    LoadProject(#[source] mabo_project::Error),
    /// The required `OUT_DIR` env var doesn't exist.
    #[error("missing OUT_DIR environment variable")]
    NoOutDir,
    /// The file name resulting from a glob pattern didn't produce a usable file path.
    #[error("failed to get the file name from a found file path")]
    NoFileName,
    /// The file name wasn't valid UTF-8.
    #[error("the file name was not encoded in valid UTF-8")]
    NonUtf8FileName,
    /// Failed to create the output directory for generated Rust source files.
    #[error("failed creating output directory at {path:?}")]
    Create {
        /// Source error of the problem.
        #[source]
        source: std::io::Error,
        /// The output directory path.
        path: PathBuf,
    },
    /// Failed to read one of the schema files.
    #[error("failed reading schema file at {file:?}")]
    Read {
        /// Source error of the problem.
        #[source]
        source: std::io::Error,
        /// The problematic file.
        file: PathBuf,
    },
    /// Failed to parse a Mabo schema.
    #[error("failed parsing schema from {file:?}:\n{report:?}")]
    Parse {
        /// Detailed report about the problem.
        report: Report,
        /// The problematic schema file.
        file: PathBuf,
    },
    /// Failed to compile a Mabo schema.
    #[error("failed compiling schema from {file:?}:\n{report:?}")]
    Compile {
        /// Detailed report about the problem.
        report: Report,
        /// The problematic schema file.
        file: PathBuf,
    },
    /// The code generator produced Rust code that isn't valid.
    #[error("failed to generate valid Rust code")]
    InvalidCode {
        /// Source error of the problem.
        #[source]
        source: syn::Error,
        /// The invalid Rust source code.
        code: String,
    },
    /// Failed to write a generated Rust source file.
    #[error("failed writing Rust source file to {file:?}")]
    Write {
        /// Source error of the problem.
        #[source]
        source: std::io::Error,
        /// The problematic file.
        file: PathBuf,
    },
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;

        write!(f, "{self}")?;

        let mut source = self.source();
        while let Some(inner) = source {
            write!(f, "\n-> {inner}")?;
            source = inner.source();
        }

        Ok(())
    }
}

/// Instance of the compiler, which is responsible to generate Rust source code from schema files.
#[derive(Default)]
pub struct Compiler {
    /// The data type to use for Mabo's `bytes` type.
    bytes_type: BytesType,
}

/// The data type to use for Mabo's `bytes` type, that is used throughout all generated schemas.
#[derive(Clone, Copy, Default)]
pub enum BytesType {
    /// Use the default `Vec<u8>` type from Rust's stdlib.
    #[default]
    VecU8,
    /// Use the [`bytes::Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) type.
    Bytes,
}

/// Additional options to adjust the behavior of the Rust code generator.
#[derive(Default)]
pub struct Opts {
    bytes_type: BytesType,
}

impl Compiler {
    /// Change the type that is used to represent Mabo `bytes` byte arrays.
    #[must_use]
    pub fn with_bytes_type(mut self, value: BytesType) -> Self {
        self.bytes_type = value;
        self
    }

    /// Compile the given list of Mabo schema files (glob patterns) into Rust source code.
    ///
    /// # Errors
    ///
    /// Will return an `Err` if any of the various cases happen, which are described in the
    /// [`Error`](enum@Error) type.
    pub fn compile(&self, manifest_dir: &str) -> Result<()> {
        init_miette();

        let project = mabo_project::load(manifest_dir).map_err(Error::LoadProject)?;
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or(Error::NoOutDir)?).join("mabo");

        fs::create_dir_all(&out_dir).map_err(|source| Error::Create {
            source,
            path: out_dir.clone(),
        })?;

        let mut inputs = Vec::new();
        let mut validated = Vec::new();

        for path in project.files {
            let input = fs::read_to_string(&path).map_err(|source| Error::Read {
                source,
                file: path.clone(),
            })?;

            inputs.push((path, input));
        }

        for (path, input) in &inputs {
            let stem = path
                .file_stem()
                .ok_or(Error::NoFileName)?
                .to_str()
                .ok_or(Error::NonUtf8FileName)?;

            let schema = Schema::parse(input, Some(path)).map_err(|e| Error::Parse {
                report: Report::new(e),
                file: path.clone(),
            })?;

            mabo_compiler::validate_schema(&schema).map_err(|e| Error::Compile {
                report: Report::new(e),
                file: path.clone(),
            })?;

            validated.push((stem, schema));
        }

        let validated = validated
            .iter()
            .map(|(name, schema)| (*name, schema))
            .collect::<Vec<_>>();

        mabo_compiler::resolve_schemas(&validated).map_err(|e| Error::Compile {
            report: Report::new(e),
            file: PathBuf::new(),
        })?;

        let opts = Opts {
            bytes_type: self.bytes_type,
        };

        for (stem, schema) in validated {
            let schema = mabo_compiler::simplify_schema(schema);
            let code = definition::compile_schema(&opts, &schema);
            let code = prettyplease::unparse(&syn::parse2(code.clone()).map_err(|source| {
                Error::InvalidCode {
                    source,
                    code: code.to_string(),
                }
            })?);

            let out_file = out_dir.join(format!("{stem}.rs"));

            fs::write(&out_file, code).map_err(|source| Error::Write {
                source,
                file: out_file,
            })?;
        }

        Ok(())
    }
}

fn init_miette() {
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
}
