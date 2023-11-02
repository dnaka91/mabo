use std::{fmt, fmt::Display, ops::Range};

use miette::{Diagnostic, NamedSource};
use thiserror::Error;

use crate::highlight;

/// Reason why type resolution failed.
#[derive(Debug)]
pub struct Error {
    pub(super) source_code: NamedSource,
    /// Cause of the failure.
    pub cause: ResolveError,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("type resolution failed")
    }
}

impl Diagnostic for Error {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.cause.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.cause.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_code)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.cause.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.cause.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.cause.diagnostic_source()
    }
}

/// Specific reason why type resolution failed, split into distinct resolution steps.
#[derive(Debug, Diagnostic, Error)]
pub enum ResolveError {
    /// Local type resolution failed.
    #[error("failed resolving type in local modules")]
    #[diagnostic(transparent)]
    Local(#[from] ResolveLocal),
    /// Import statement resolution failed.
    #[error("failed resolving import statement")]
    #[diagnostic(transparent)]
    Import(#[from] ResolveImport),
    /// Remote (types in another schema) type resolution failed.
    #[error("failed resolving type in remote modules")]
    #[diagnostic(transparent)]
    Remote(#[from] Box<ResolveRemote>),
}

impl From<ResolveRemote> for ResolveError {
    fn from(value: ResolveRemote) -> Self {
        Self::Remote(value.into())
    }
}

/// Failed to resolve the type within a schema's root or one of its submodules.
#[derive(Debug, Diagnostic, Error)]
pub enum ResolveLocal {
    /// The referenced submodule doesn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    /// The referenced definition in the root or a submodule doesn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    /// The amount of generics between declaration and use side doesn't match.
    #[error(transparent)]
    #[diagnostic(transparent)]
    GenericsCount(#[from] GenericsCount),
    /// The referenced definition can't be used as type.
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] InvalidKind),
}

/// The referenced (sub)module wasn't found in the schema.
#[derive(Debug, Diagnostic, Error)]
#[error("module {} not found", highlight::value(name))]
#[diagnostic(help("the resolution stopped at module path {}", highlight::value(path)))]
pub struct MissingModule {
    /// Name of the missing module.
    pub name: String,
    /// Path of modules at which the resolution failed.
    pub path: String,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// The referenced type wasn't found in the schema root or submodule.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition {} not found in module {}",
    highlight::value(name),
    highlight::value(path)
)]
pub struct MissingDefinition {
    /// Name of the missing type.
    pub name: String,
    /// Path of the resolved module where resolution failed.
    pub path: String,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// The referenced type was found but the amount of generic type parameters didn't match.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "the definition has {} generics but the use side has {}",
    highlight::value(definition),
    highlight::value(usage)
)]
#[diagnostic(help("the amount of generics must always match"))]
pub struct GenericsCount {
    /// Amount of generics on the declaration side.
    pub definition: usize,
    /// Amount of generics on the use side.
    pub usage: usize,
    #[label("declared here")]
    pub(super) declared: Range<usize>,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// The referenced definition was found but it's not a type that can be referenced.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition found, but a {} can't be referenced",
    highlight::sample(kind)
)]
#[diagnostic(help("only struct and enum definitions can be used"))]
pub struct InvalidKind {
    /// The kind of definition that was found.
    pub kind: &'static str,
    #[label("declared here")]
    pub(super) declared: Range<usize>,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// Failed to resolve an import of another schema.
#[derive(Debug, Diagnostic, Error)]
pub enum ResolveImport {
    /// The referenced schema doesn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingSchema(#[from] MissingSchema),
    /// The referenced module inside the schema doesn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    /// The referenced type inside the module deosn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    /// The referenced definition can't be used as type.
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] InvalidKind),
}

/// The referenced schema wasn't found in the list of available schemas.
#[derive(Debug, Diagnostic, Error)]
#[error("schema {} not found", highlight::value(name))]
pub struct MissingSchema {
    /// Name of the missing schema.
    pub name: String,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// Failed to resolve a type in another schema.
#[derive(Debug, Diagnostic, Error)]
pub enum ResolveRemote {
    /// No matching import for the type exists.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingImport(#[from] MissingImport),
    /// The referenced module inside the schema doesn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    /// The referenced type inside the module deosn't exist.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    /// The amount of generics between declaration and use side doesn't match.
    #[error(transparent)]
    #[diagnostic(transparent)]
    GenericsCount(#[from] RemoteGenericsCount),
    /// The referenced definition can't be used as type.
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] RemoteInvalidKind),
}

/// None of the existing imports match for the referenced type.
#[derive(Debug, Diagnostic, Error)]
#[error("missing import for type {}", highlight::value(ty))]
pub struct MissingImport {
    /// Name of the type.
    pub ty: String,
    #[label("used here")]
    pub(super) used: Range<usize>,
}

/// Like [`GenericsCount`], the amount of generics between declaration side and use side didn't
/// match, but split into two separate errors to allow error reporting in separate schema files.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "the use side has {} generic(s), mismatching with the declaration",
    highlight::value(amount)
)]
#[diagnostic(help("the amount of generics must always match"))]
pub struct RemoteGenericsCount {
    /// Amount of generics on the use side.
    pub amount: usize,
    #[label("used here")]
    pub(super) used: Range<usize>,
    /// Error for the declaration side.
    #[related]
    pub declaration: [RemoteGenericsCountDeclaration; 1],
}

/// Declaration side error for the [`RemoteGenericsCount`].
#[derive(Debug, Diagnostic, Error)]
#[error(
    "the declaration has {} generic(s), mismatching with the use side",
    highlight::value(amount)
)]
pub struct RemoteGenericsCountDeclaration {
    /// Amount of generics on the declaration side.
    pub amount: usize,
    #[source_code]
    pub(super) source_code: NamedSource,
    #[label("declared here")]
    pub(super) used: Range<usize>,
}

/// Like [`InvalidKind`], the referenced definition was found yet is not a type that can be
/// referenced, but split into to separate errors to allow error reporting in separate schema files.
#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition found, but a {} can't be referenced",
    highlight::sample(kind)
)]
#[diagnostic(help("only struct and enum definitions can be used"))]
pub struct RemoteInvalidKind {
    /// The kind of definition that was found.
    pub kind: &'static str,
    #[label("used here")]
    pub(super) used: Range<usize>,
    /// Error for the declaration side.
    #[related]
    pub declaration: [RemoteInvalidKindDeclaration; 1],
}

/// Declaration side error for the [`RemoteInvalidKind`].
#[derive(Debug, Diagnostic, Error)]
#[error(
    "the definition is a {}, which can't be referenced",
    highlight::sample(kind)
)]
pub struct RemoteInvalidKindDeclaration {
    /// The kind of definition that is declared.
    pub kind: &'static str,
    #[source_code]
    pub(super) source_code: NamedSource,
    #[label("declared here")]
    pub(super) used: Range<usize>,
}
