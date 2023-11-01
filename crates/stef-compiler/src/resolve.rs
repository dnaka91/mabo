use std::{collections::HashMap, ops::Range};

use miette::Diagnostic;
use stef_parser::{
    DataType, Definition, ExternalType, Fields, Generics, Import, Name, Schema, Spanned,
};
use thiserror::Error;

use crate::highlight;

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveError {
    #[error("failed resolving type in local modules")]
    #[diagnostic(transparent)]
    Local(#[from] ResolveLocal),
    #[error("failed resolving import statement")]
    #[diagnostic(transparent)]
    Import(#[from] ResolveImport),
    #[error("failed resolving type in remote modules")]
    #[diagnostic(transparent)]
    Remote(#[from] ResolveRemote),
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveLocal {
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    #[error(transparent)]
    #[diagnostic(transparent)]
    GenericsCount(#[from] GenericsCount),
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] InvalidKind),
}

#[derive(Debug, Diagnostic, Error)]
#[error("module {} not found", highlight::value(name))]
#[diagnostic(help("the resolution stopped at module path {}", highlight::value(path)))]
pub struct MissingModule {
    pub name: String,
    pub path: String,
    #[label("used here")]
    pub used: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition {} not found in module {}",
    highlight::value(name),
    highlight::value(path)
)]
pub struct MissingDefinition {
    pub name: String,
    pub path: String,
    #[label("used here")]
    pub used: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "the definition has {} generics but the use side has {}",
    highlight::value(definition),
    highlight::value(usage)
)]
#[diagnostic(help("the amount of generics must always match"))]
pub struct GenericsCount {
    pub definition: usize,
    pub usage: usize,
    #[label("declared here")]
    pub declared: Range<usize>,
    #[label("used here")]
    pub used: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition found, but a {} can't be referenced",
    highlight::sample(kind)
)]
#[diagnostic(help("only struct and enum definitions can be used"))]
pub struct InvalidKind {
    pub kind: &'static str,
    #[label("declared here")]
    pub declared: Range<usize>,
    #[label("used here")]
    pub used: Range<usize>,
}

pub(crate) struct Module<'a> {
    /// Name of this module.
    pub name: &'a str,
    /// Full path from the root (the schema) till here.
    path: String,
    path2: Vec<&'a str>,
    /// List of imports declared in the module.
    imports: Vec<&'a Import<'a>>,
    /// List of types that are declared in this module.
    types: Vec<Type<'a>>,
    /// Directly submodules located in this module.
    modules: HashMap<&'a str, Module<'a>>,
    definitions: &'a [Definition<'a>],
}

struct Type<'a> {
    kind: TypeKind,
    name: Name<'a>,
}

enum TypeKind {
    Struct { generics: usize },
    Enum { generics: usize },
    Alias,
    Const,
}

pub(crate) enum ResolvedImport<'a> {
    Module(&'a Module<'a>),
    Type { name: &'a Name<'a>, generics: usize },
}

impl<'a> Module<'a> {
    fn resolve_local(&self, ty: &ExternalType<'_>) -> Result<(), ResolveLocal> {
        let module = if ty.path.is_empty() {
            self
        } else {
            ty.path.iter().try_fold(self, |module, name| {
                module.modules.get(name.get()).ok_or_else(|| MissingModule {
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
            .ok_or_else(|| MissingDefinition {
                name: ty.name.get().to_owned(),
                path: module.path.clone(),
                used: ty.name.span().into(),
            })?;

        match definition.kind {
            TypeKind::Struct { generics } | TypeKind::Enum { generics }
                if generics != ty.generics.len() =>
            {
                Err(GenericsCount {
                    definition: generics,
                    usage: ty.generics.len(),
                    declared: definition.name.span().into(),
                    used: ty.name.span().into(),
                }
                .into())
            }
            TypeKind::Alias => Err(InvalidKind {
                kind: "type alias",
                declared: definition.name.span().into(),
                used: ty.name.span().into(),
            }
            .into()),
            TypeKind::Const => Err(InvalidKind {
                kind: "constant",
                declared: definition.name.span().into(),
                used: ty.name.span().into(),
            }
            .into()),
            _ => Ok(()),
        }
    }

    fn resolve_import(&self, import: &Import<'_>) -> Result<ResolvedImport<'_>, ResolveImport> {
        let module = if import.segments.len() < 2 {
            self
        } else {
            import
                .segments
                .iter()
                .skip(1)
                .try_fold(self, |module, name| {
                    module.modules.get(name.get()).ok_or_else(|| MissingModule {
                        name: name.get().to_owned(),
                        path: module.path.clone(),
                        used: name.span().into(),
                    })
                })?
        };

        if let Some(element) = import.element.as_ref() {
            let definition = module
                .types
                .iter()
                .find(|type_def| type_def.name.get() == element.get())
                .ok_or_else(|| MissingDefinition {
                    name: element.get().to_owned(),
                    path: module.path.clone(),
                    used: element.span().into(),
                })?;

            match definition.kind {
                TypeKind::Alias => Err(InvalidKind {
                    kind: "alias",
                    declared: definition.name.span().into(),
                    used: element.span().into(),
                }
                .into()),
                TypeKind::Const => Err(InvalidKind {
                    kind: "const",
                    declared: definition.name.span().into(),
                    used: element.span().into(),
                }
                .into()),
                TypeKind::Struct { generics } | TypeKind::Enum { generics } => {
                    Ok(ResolvedImport::Type {
                        name: &definition.name,
                        generics,
                    })
                }
            }
        } else {
            Ok(ResolvedImport::Module(module))
        }
    }

    fn resolve_remote(&self, ty: &ExternalType<'_>) -> Result<(), ResolveRemote> {
        let module = if ty
            .path
            .first()
            .is_some_and(|first| first.get() == self.name)
        {
            self
        } else {
            ty.path.iter().try_fold(self, |module, name| {
                module.modules.get(name.get()).ok_or_else(|| MissingModule {
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
            .ok_or_else(|| MissingDefinition {
                name: ty.name.get().to_owned(),
                path: module.path.clone(),
                used: ty.name.span().into(),
            })?;

        match definition.kind {
            TypeKind::Struct { generics } | TypeKind::Enum { generics }
                if generics != ty.generics.len() =>
            {
                Err(RemoteGenericsCount {
                    definition: generics,
                    usage: ty.generics.len(),
                    used: ty.name.span().into(),
                }
                .into())
            }
            TypeKind::Alias => Err(RemoteInvalidKind {
                kind: "type alias",
                used: ty.name.span().into(),
            }
            .into()),
            TypeKind::Const => Err(RemoteInvalidKind {
                kind: "constant",
                used: ty.name.span().into(),
            }
            .into()),
            _ => Ok(()),
        }
    }
}

pub(crate) struct LocallyMissingType<'a> {
    pub external: &'a ExternalType<'a>,
    pub error: ResolveLocal,
}

pub(crate) fn resolve_module_types<'a>(
    module: &'a Module<'_>,
    missing: &mut Vec<LocallyMissingType<'a>>,
) {
    fn is_generic(external: &ExternalType<'_>, generics: &Generics<'_>) -> bool {
        external.generics.is_empty()
            && external.path.is_empty()
            && generics
                .0
                .iter()
                .any(|gen| gen.get() == external.name.get())
    }

    fn resolve<'a>(
        missing: &mut Vec<LocallyMissingType<'a>>,
        ty: &'a DataType<'_>,
        generics: &Generics<'_>,
        module: &'a Module<'_>,
    ) {
        visit_externals(ty, &mut |external| {
            if !is_generic(external, generics) {
                if let Err(e) = module.resolve_local(external) {
                    missing.push(LocallyMissingType { external, error: e });
                }
            }
        });
    }

    for def in module.definitions {
        match def {
            Definition::Struct(s) => match &s.fields {
                Fields::Named(named) => {
                    for field in named {
                        resolve(missing, &field.ty, &s.generics, module);
                    }
                }
                Fields::Unnamed(unnamed) => {
                    for field in unnamed {
                        resolve(missing, &field.ty, &s.generics, module);
                    }
                }
                Fields::Unit => {}
            },
            Definition::Enum(e) => {
                for variant in &e.variants {
                    match &variant.fields {
                        Fields::Named(named) => {
                            for field in named {
                                resolve(missing, &field.ty, &e.generics, module);
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            for field in unnamed {
                                resolve(missing, &field.ty, &e.generics, module);
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
        resolve_module_types(module, missing);
    }
}

pub(crate) fn resolve_types<'a>(name: &'a str, value: &'a Schema<'_>) -> Module<'a> {
    visit_module_tree(name, &[], &value.definitions)
}

/// Build up modules from the given one all the way down to all submodules.
///
/// This builds a tree structure of elements defined in each module, so they can be looked up in a
/// 2nd step to ensure all used types are actually available and correct.
fn visit_module_tree<'a>(
    name: &'a str,
    path: &[&'a str],
    defs: &'a [Definition<'_>],
) -> Module<'a> {
    let path = {
        let mut new = Vec::with_capacity(path.len());
        new.extend(path);
        new.push(name);
        new
    };

    let mut module = Module {
        name,
        path: path.join("::"),
        path2: path,
        imports: Vec::new(),
        types: Vec::new(),
        modules: HashMap::new(),
        definitions: defs,
    };

    for def in defs {
        match def {
            Definition::Module(m) => {
                module.modules.insert(
                    m.name.get(),
                    visit_module_tree(m.name.get(), &module.path2, &m.definitions),
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
            Definition::TypeAlias(a) => module.types.push(Type {
                kind: TypeKind::Alias,
                name: a.name.clone(),
            }),
            Definition::Const(c) => module.types.push(Type {
                kind: TypeKind::Const,
                name: c.name.clone(),
            }),
            Definition::Import(i) => module.imports.push(i),
        }
    }

    module
}

fn visit_externals<'a>(value: &'a DataType<'_>, visit: &mut impl FnMut(&'a ExternalType<'_>)) {
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
            visit_externals(ty, visit);
        }
        DataType::HashMap(kv) => {
            visit_externals(&kv.0, visit);
            visit_externals(&kv.1, visit);
        }
        DataType::Tuple(types) => {
            for ty in types {
                visit_externals(ty, visit);
            }
        }
        DataType::External(ty) => {
            visit(ty);

            for ty in &ty.generics {
                visit_externals(ty, visit);
            }
        }
    }
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveImport {
    #[error("schema {} not found", highlight::value(name))]
    MissingSchema {
        name: String,
        #[label("used here")]
        used: Range<usize>,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] InvalidKind),
}

pub(crate) fn resolve_module_imports<'a>(
    module: &Module<'_>,
    schemas: &'a [(&str, Module<'_>)],
) -> Result<Vec<ResolvedImport<'a>>, ResolveImport> {
    module
        .imports
        .iter()
        .map(|import| {
            let root = &import.segments[0];
            let schema = schemas
                .iter()
                .find_map(|(name, schema)| (*name == root.get()).then_some(schema))
                .ok_or_else(|| ResolveImport::MissingSchema {
                    name: root.get().to_owned(),
                    used: root.span().into(),
                })?;

            schema.resolve_import(import)
        })
        .collect()
}

#[derive(Debug, Diagnostic, Error)]
pub enum ResolveRemote {
    #[error("missing import for type {}", highlight::value(ty))]
    MissingImport {
        ty: String,
        #[label("used here")]
        used: Range<usize>,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingModule(#[from] MissingModule),
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingDefinition(#[from] MissingDefinition),
    #[error(transparent)]
    #[diagnostic(transparent)]
    GenericsCount(#[from] RemoteGenericsCount),
    #[error(transparent)]
    #[diagnostic(transparent)]
    InvalidKind(#[from] RemoteInvalidKind),
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "the definition has {} generics but the use side has {}",
    highlight::value(definition),
    highlight::value(usage)
)]
#[diagnostic(help("the amount of generics must always match"))]
pub struct RemoteGenericsCount {
    pub definition: usize,
    pub usage: usize,
    #[label("used here")]
    pub used: Range<usize>,
}

#[derive(Debug, Diagnostic, Error)]
#[error(
    "definition found, but a {} can't be referenced",
    highlight::sample(kind)
)]
#[diagnostic(help("only struct and enum definitions can be used"))]
pub struct RemoteInvalidKind {
    pub kind: &'static str,
    #[label("used here")]
    pub used: Range<usize>,
}

pub(crate) fn resolve_type_remotely(
    ty: LocallyMissingType<'_>,
    imports: &[ResolvedImport<'_>],
) -> Result<(), super::Error> {
    if imports.is_empty() {
        return Err(ty.error.into());
    } else if let Some(name) = ty.external.path.first() {
        let module = imports.iter().find_map(|import| match import {
            ResolvedImport::Module(module) => (module.name == name.get()).then_some(module),
            ResolvedImport::Type { .. } => None,
        });

        match module {
            Some(module) => module.resolve_remote(ty.external)?,
            None => {
                return Err(ResolveRemote::MissingImport {
                    ty: format!(
                        "{}{}",
                        ty.external
                            .path
                            .iter()
                            .fold(String::new(), |mut acc, part| {
                                acc.push_str(part.get());
                                acc.push_str("::");
                                acc
                            }),
                        ty.external.name
                    ),
                    used: ty.external.name.span().into(),
                }
                .into());
            }
        }
    } else {
        let found = imports.iter().find_map(|import| match import {
            ResolvedImport::Module(_) => None,
            ResolvedImport::Type { name, generics } => {
                (name.get() == ty.external.name.get()).then_some((name, *generics))
            }
        });

        if let Some((_name, generics)) = found {
            if generics == ty.external.generics.len() {
                return Ok(());
            }

            return Err(ResolveRemote::GenericsCount(RemoteGenericsCount {
                definition: generics,
                usage: ty.external.generics.len(),
                used: ty.external.name.span().into(),
            })
            .into());
        }

        return Err(ResolveRemote::MissingImport {
            ty: ty.external.name.get().to_owned(),
            used: ty.external.name.span().into(),
        }
        .into());
    }

    Err(ty.error.into())
}
