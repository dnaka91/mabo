use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{DataType, Definition, ExternalType, Fields, Generics, Name, Schema, Spanned};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveError {
    #[error("failed resolving type in local modules")]
    #[diagnostic(transparent)]
    Local(#[from] ResolveLocal),
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveLocal {
    #[error("module {name} not found")]
    #[diagnostic(help("the resolution stopped at module path {path}"))]
    MissingModule {
        name: String,
        path: String,
        #[label("used here")]
        used: Range<usize>,
    },
    #[error("definition {name} not found in module {path}")]
    MissingDefinition {
        name: String,
        path: String,
        #[label("used here")]
        used: Range<usize>,
    },
    #[error("the definition has {definition} generics but the use side has {usage}")]
    #[diagnostic(help("the amount of generics must always match"))]
    GenericsCount {
        definition: usize,
        usage: usize,
        #[label("declared here")]
        declared: Range<usize>,
        #[label("used here")]
        used: Range<usize>,
    },
    #[error("definition found, but a {kind} can't be referenced")]
    #[diagnostic(help("only struct and enum definitions can be used"))]
    InvalidKind {
        kind: &'static str,
        #[label("declared here")]
        declared: Range<usize>,
        #[label("used here")]
        used: Range<usize>,
    },
}

pub struct Module<'a> {
    /// Name of this module.
    pub name: &'a str,
    /// Full path from the root (the schema) till here.
    path: String,
    /// List of types that are declared in this module.
    types: Vec<Type<'a>>,
    /// Directly submodules located in this module.
    modules: HashMap<&'a str, Module<'a>>,
    definitions: &'a [Definition<'a>],
}

pub struct Type<'a> {
    kind: TypeKind,
    name: Name<'a>,
}

pub enum TypeKind {
    Struct { generics: usize },
    Enum { generics: usize },
    Const,
}

impl<'a> Module<'a> {
    pub fn resolve_local(&self, ty: &ExternalType<'_>) -> Result<(), ResolveLocal> {
        let module = if ty.path.is_empty() {
            self
        } else {
            ty.path.iter().try_fold(self, |module, name| {
                module
                    .modules
                    .get(name.get())
                    .ok_or_else(|| ResolveLocal::MissingModule {
                        name: name.get().to_owned(),
                        path: module.path.clone(),
                        used: ty.name.span().into(),
                    })
            })?
        };

        let definition = module
            .types
            .iter()
            .find(|type_def| type_def.name.get() == ty.name.get())
            .ok_or_else(|| ResolveLocal::MissingDefinition {
                name: ty.name.get().to_owned(),
                path: module.path.clone(),
                used: ty.name.span().into(),
            })?;

        match definition.kind {
            TypeKind::Struct { generics } | TypeKind::Enum { generics }
                if generics != ty.generics.len() =>
            {
                Err(ResolveLocal::GenericsCount {
                    definition: generics,
                    usage: ty.generics.len(),
                    declared: definition.name.span().into(),
                    used: ty.name.span().into(),
                })
            }
            TypeKind::Const => Err(ResolveLocal::InvalidKind {
                kind: "const",
                declared: definition.name.span().into(),
                used: ty.name.span().into(),
            }),
            _ => Ok(()),
        }
    }
}

pub(crate) fn resolve_module_definitions(module: &Module<'_>) -> Result<(), ResolveError> {
    fn resolve(
        ty: &DataType<'_>,
        generics: &Generics<'_>,
        module: &Module<'_>,
    ) -> Result<(), ResolveLocal> {
        visit_externals(ty, &mut |external| {
            if external.generics.is_empty()
                && external.path.is_empty()
                && generics
                    .0
                    .iter()
                    .any(|gen| gen.get() == external.name.get())
            {
                Ok(())
            } else {
                module.resolve_local(external)
            }
        })
    }

    for def in module.definitions {
        match def {
            Definition::Struct(s) => match &s.fields {
                Fields::Named(named) => {
                    for field in named {
                        resolve(&field.ty, &s.generics, module)?;
                    }
                }
                Fields::Unnamed(unnamed) => {
                    for field in unnamed {
                        resolve(&field.ty, &s.generics, module)?;
                    }
                }
                Fields::Unit => {}
            },
            Definition::Enum(e) => {
                for variant in &e.variants {
                    match &variant.fields {
                        Fields::Named(named) => {
                            for field in named {
                                resolve(&field.ty, &e.generics, module)?;
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            for field in unnamed {
                                resolve(&field.ty, &e.generics, module)?;
                            }
                        }
                        Fields::Unit => {}
                    }
                }
            }
            _ => {}
        }
    }

    for module in module.modules.values() {
        resolve_module_definitions(module)?;
    }

    Ok(())
}

pub(crate) fn resolve_types<'a>(name: &'a str, value: &'a Schema<'_>) -> Module<'a> {
    visit_module_tree(name, "", &value.definitions)
}

/// Build up modules from the given one all the way down to all submodules.
///
/// This builds a tree structure of elements defined in each module, so they can be looked up in a
/// 2nd step to ensure all used types are actually available and correct.
fn visit_module_tree<'a>(name: &'a str, path: &'_ str, defs: &'a [Definition<'_>]) -> Module<'a> {
    let mut module = Module {
        name,
        path: format!("{path}::{name}"),
        types: Vec::new(),
        modules: HashMap::new(),
        definitions: defs,
    };

    for def in defs {
        match def {
            Definition::Module(m) => {
                module.modules.insert(
                    m.name.get(),
                    visit_module_tree(m.name.get(), &module.path, &m.definitions),
                );
            }
            Definition::Struct(s) => module.types.push(Type {
                kind: TypeKind::Struct {
                    generics: s.generics.0.len(),
                },
                name: s.name.clone(),
            }),
            Definition::Enum(e) => module.types.push(Type {
                kind: TypeKind::Enum {
                    generics: e.generics.0.len(),
                },
                name: e.name.clone(),
            }),
            Definition::Const(c) => module.types.push(Type {
                kind: TypeKind::Const,
                name: c.name.clone(),
            }),
            _ => {}
        }
    }

    module
}

fn visit_externals<E, F>(value: &DataType<'_>, visit: &mut F) -> Result<(), E>
where
    F: FnMut(&ExternalType<'_>) -> Result<(), E>,
{
    match value {
        DataType::Bool
        | DataType::U8
        | DataType::U16
        | DataType::U32
        | DataType::U64
        | DataType::U128
        | DataType::I8
        | DataType::I16
        | DataType::I32
        | DataType::I64
        | DataType::I128
        | DataType::F32
        | DataType::F64
        | DataType::String
        | DataType::StringRef
        | DataType::Bytes
        | DataType::BytesRef
        | DataType::NonZero(_)
        | DataType::BoxString
        | DataType::BoxBytes => {}
        DataType::Vec(ty)
        | DataType::HashSet(ty)
        | DataType::Option(ty)
        | DataType::Array(ty, _) => {
            visit_externals(ty, visit)?;
        }
        DataType::HashMap(kv) => {
            visit_externals(&kv.0, visit)?;
            visit_externals(&kv.1, visit)?;
        }
        DataType::Tuple(types) => {
            for ty in types {
                visit_externals(ty, visit)?;
            }
        }
        DataType::External(ty) => {
            visit(ty)?;

            for ty in &ty.generics {
                visit_externals(ty, visit)?;
            }
        }
    }

    Ok(())
}
