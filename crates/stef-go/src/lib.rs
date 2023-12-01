//! Schema to source code converter for the _Go_ programming language.

pub use definition::render_schema;

mod decode;
mod definition;
mod encode;

/// Options for the code generator that can modify the way the code is generated.
#[derive(Default)]
pub struct Opts<'a> {
    /// Name of the package for the root schema. Eventual sub-modules will have their package name
    /// the schema's module name.
    pub package: &'a str,
}

/// The output of generating converting a schema file into one or more Go source code files. The
/// files' content solely resides in this structure and still needs to be saved to the file system.
///
/// As Go doesn't allow to directly define modules within a single file, a tree structure is formed
/// with each direct module located in the current module, their direct modules being located in
/// them, and so on.
#[derive(Debug)]
pub struct Output<'a> {
    /// Name of this output as derived from the module name.
    pub name: &'a str,
    /// Final Go source code output of the module file.
    pub content: String,
    /// All modules that were defined as direct children of this module.
    pub modules: Vec<Output<'a>>,
}
