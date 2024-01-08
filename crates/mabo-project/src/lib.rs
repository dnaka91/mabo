//! Loading and resolution of `Mabo.toml` project files.
//!
//! This crate defines the structs for the contents of project files, as well as common resolution
//! logic like finding all projects in a folder or resolving schema search patterns into absolute
//! file paths.

use std::{
    fs,
    path::{Path, PathBuf},
};

use globset::{GlobBuilder, GlobSetBuilder};
use ignore::{Walk, WalkBuilder};
use serde::Deserialize;

mod de;

/// Shorthand for the standard result type, that defaults to the crate level's [`Error`] type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can happen when loading Mabo projects.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// One of the glob patterns is invalid.
    #[error("failed to parse the glob pattern {glob:?}")]
    Pattern {
        /// Source error of the problem.
        #[source]
        source: globset::Error,
        /// The problematic pattern.
        glob: String,
    },
    /// The glob patterns are valid but couldn't be combined into a single set.
    #[error("failed to combine the glob patterns into a set")]
    PatternSet(#[source] globset::Error),
    /// Failed to iterate over the matching files for a pattern.
    #[error("failed to walk over the file tree")]
    Walk(#[source] ignore::Error),
    /// Failed to read a Mabo project file.
    #[error("failed reading project file at {file:?}")]
    Read {
        /// Source error of the problem.
        #[source]
        source: std::io::Error,
        /// The problematic file.
        file: PathBuf,
    },
    /// Failed to parse a Mabo project file.
    #[error("failed parsing project file {file:?}")]
    Parse {
        /// Source error of the problem.
        #[source]
        source: Box<toml::de::Error>,
        /// The problematic schema file.
        file: PathBuf,
    },
    /// Failed to strip the base path from a found schema file.
    #[error("failed to turn an absolute path into a relative one")]
    StripPrefix(#[source] std::path::StripPrefixError),
}

/// Parsed `Mabo.toml` project file.
#[derive(Debug, Deserialize)]
pub struct ProjectFile {
    /// The package that defines the content of this project.
    pub package: Package,
}

///  Single named collection of schema files that form a package.
#[derive(Debug, Deserialize)]
pub struct Package {
    /// The package name is an identifier used to refer to the package.
    pub name: String,
    /// The description is a short description about the package.
    pub description: Option<String>,
    /// The license defines the software license that this package is released under. It can be a
    /// single [SPDX](https://spdx.dev/) license, or multiple combined with `AND` and `OR` into an
    /// expression.
    ///
    /// See the [SPDX Specification](https://spdx.github.io/spdx-spec/v2.3/) for more details about
    /// the exact expression syntax.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [package]
    /// # ...
    /// license = "MIT OR Apache-2.0"
    /// ```
    #[serde(default, deserialize_with = "de::spdx_expression_opt")]
    pub license: Option<spdx::Expression>,
    /// List of files that make up the schema package. These are not regular file paths but glob
    /// patterns, meaning that file trees can be defined in a consise way like `schemas/**/*.mabo`.
    ///
    /// Regardless of the [glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)) defined
    /// the final file list is always filtered by the `.mabo` file extension.
    pub files: Vec<String>,
}

/// Single project that was loaded from a `Mabo.toml` file and all files and additional information
/// that comes with it.
#[derive(Debug)]
pub struct Project {
    /// Parsed content of the `Mabo.toml` project file.
    pub project_file: ProjectFile,
    /// Location of the `Mabo.toml` project file.
    pub project_path: PathBuf,
    /// Resolved final list of files to process.
    pub files: Vec<PathBuf>,
}

/// Search through a project folder and load all possible Mabo project contained within.
///
/// # Errors
///
/// Will return `Err` in case of any I/O failure, missing files or an invalid project file format.
pub fn discover(base: impl AsRef<Path>) -> Result<Vec<Project>> {
    let pattern = GlobBuilder::new("**/Mabo.toml")
        .literal_separator(true)
        .build()
        .map_err(|source| Error::Pattern {
            source,
            glob: "**/Mabo.toml".to_owned(),
        })?
        .compile_matcher();

    Walk::new(base)
        .filter_map(Result::ok)
        .filter(|f| f.file_type().map_or(false, |ty| ty.is_file()) && pattern.is_match(f.path()))
        .filter_map(|file| {
            file.path()
                .parent()
                .map(|base| load_project(base, file.path()))
        })
        .collect()
}

/// Load a single `Mabo.toml` project, if it's known that there is only a single one possible.
///
/// This is usually the case when loaded from a single project of a programming language, like a
/// Rust project with a `build.rs` build script.
///
/// # Errors
///
/// Will return `Err` in case of any I/O failure, missing files or an invalid project file format.
pub fn load(base: impl AsRef<Path>) -> Result<Project> {
    let base = base.as_ref();
    let file = base.join("Mabo.toml");

    load_project(base, &file)
}

fn load_project(base: &Path, file: &Path) -> Result<Project> {
    let project_file = fs::read_to_string(file).map_err(|source| Error::Read {
        source,
        file: file.to_owned(),
    })?;
    let project_file =
        toml::from_str::<ProjectFile>(&project_file).map_err(|source| Error::Parse {
            source: source.into(),
            file: file.to_owned(),
        })?;

    let files = collect_files(base, &project_file.package.files)?;

    Ok(Project {
        project_file,
        project_path: file.to_owned(),
        files,
    })
}

fn collect_files(base: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let walk = WalkBuilder::new(base)
        .follow_links(true)
        .same_file_system(true)
        .skip_stdout(true)
        .build();

    let patterns = patterns
        .iter()
        .map(|pattern| {
            GlobBuilder::new(pattern)
                .literal_separator(true)
                .empty_alternates(true)
                .build()
                .map_err(|source| Error::Pattern {
                    source,
                    glob: pattern.to_owned(),
                })
        })
        .try_fold(GlobSetBuilder::new(), |mut set, pattern| {
            set.add(pattern?);
            Ok(set)
        })?
        .build()
        .map_err(Error::PatternSet)?;

    let mut files = Vec::new();

    for entry in walk {
        let entry = entry.map_err(Error::Walk)?;
        let path = entry
            .path()
            .strip_prefix(base)
            .map_err(Error::StripPrefix)?;

        if patterns.is_match(path) && path.extension().map_or(false, |ext| ext == "mabo") {
            files.push(entry.into_path());
        }
    }

    Ok(files)
}
