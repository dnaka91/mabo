//! Ensure all referenced types within a schema itself, aswell as between schemas exist and are
//! correct.

use mabo_parser::{
    punctuated::Punctuated,
    token::{self, Punctuation},
    DataType, Definition, ExternalType, Fields, Generics, Import, Name, Schema, Spanned, Type,
};
use miette::NamedSource;
use rustc_hash::FxHashMap;

pub use self::error::{
    Error, GenericsCount, InvalidKind, MissingDefinition, MissingImport, MissingModule,
    MissingSchema, RemoteGenericsCount, RemoteGenericsCountDeclaration, RemoteInvalidKind,
    RemoteInvalidKindDeclaration, ResolveError, ResolveImport, ResolveLocal, ResolveRemote,
};

mod error;

/// Ensure all referenced types in the schema definitions exist and are valid.
///
/// This validation happens in three distinct steps:
/// - First, each schema is checked individually, trying to resolve types from submodules. Any
///   not-found types are collected for later checks against external schemas.
/// - Then, the imports in each schema are checked to point to an existing type or module in another
///   schema.
/// - Lastly, the not-found types from the first steps are checked for in the other schemas by
///   utilizing the imports from the second step.
///
/// # Errors
///
/// Will return `Err` if any of the resolution steps fails.
pub fn schemas(values: &[(&str, &Schema<'_>)]) -> Result<(), Error> {
    let modules = values
        .iter()
        .map(|(name, schema)| (*name, resolve_types(name, schema)))
        .collect::<Vec<_>>();

    for (schema, module) in modules
        .iter()
        .enumerate()
        .map(|(i, (_, module))| (values[i].1, module))
    {
        let mut missing = Vec::new();
        resolve_module_types(module, &mut missing);

        let imports = resolve_module_imports(module, &modules).map_err(|e| Error {
            source_code: NamedSource::new(
                schema
                    .path
                    .as_ref()
                    .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                schema.source.to_owned(),
            ),
            cause: ResolveError::Import(e.into()),
        })?;

        for ty in missing {
            resolve_type_remotely(ty, &imports).map_err(|e| Error {
                source_code: NamedSource::new(
                    schema
                        .path
                        .as_ref()
                        .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                    schema.source.to_owned(),
                ),
                cause: e,
            })?;
        }
    }

    Ok(())
}

pub(crate) struct Module<'a> {
    /// Name of this module.
    pub name: &'a str,
    /// Reference back into the schema.
    schema: &'a Schema<'a>,
    /// Full path from the root (the schema) till here, split into individual components.
    path: Vec<&'a str>,
    /// List of imports declared in the module.
    imports: Vec<&'a Import<'a>>,
    /// List of types that are declared in this module.
    types: Vec<Declaration<'a>>,
    /// Direct submodules located in this module.
    modules: FxHashMap<&'a str, Module<'a>>,
    /// List of original definitions.
    definitions: &'a [Definition<'a>],
}

impl Module<'_> {
    fn path_to_string(&self) -> String {
        self.path.join("::")
    }
}

/// Simplified declaration with just enough information to check whether declaration and use site
/// are compatible with another.
struct Declaration<'a> {
    kind: DeclarationKind,
    name: Name<'a>,
}

enum DeclarationKind {
    Struct { generics: usize },
    Enum { generics: usize },
    Alias,
    Const,
}

impl DeclarationKind {
    fn as_str(&self) -> &'static str {
        match self {
            DeclarationKind::Struct { .. } => "struct",
            DeclarationKind::Enum { .. } => "enum",
            DeclarationKind::Alias => "type alias",
            DeclarationKind::Const => "constant",
        }
    }
}

pub(crate) enum ResolvedImport<'a> {
    Module(&'a Module<'a>),
    Type {
        schema: &'a Schema<'a>,
        name: &'a Name<'a>,
        generics: usize,
    },
}

impl Module<'_> {
    fn resolve_local(&self, ty: &ExternalType<'_>) -> Result<(), ResolveLocal> {
        let module = if ty.path.is_empty() {
            self
        } else {
            ty.path.iter().try_fold(self, |module, (name, _)| {
                module.modules.get(name.get()).ok_or_else(|| MissingModule {
                    name: name.get().to_owned(),
                    path: module.path_to_string(),
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
                path: module.path_to_string(),
                used: ty.name.span().into(),
            })?;

        match definition.kind {
            DeclarationKind::Struct { generics } | DeclarationKind::Enum { generics }
                if generics != ty.generics.as_ref().map_or(0, Punctuated::len) =>
            {
                Err(GenericsCount {
                    definition: generics,
                    usage: ty.generics.as_ref().map_or(0, Punctuated::len),
                    declared: definition.name.span().into(),
                    used: ty.name.span().into(),
                }
                .into())
            }
            DeclarationKind::Alias | DeclarationKind::Const => Err(InvalidKind {
                kind: definition.kind.as_str(),
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
                        path: module.path_to_string(),
                        used: name.span().into(),
                    })
                })?
        };

        if let Some((_, element)) = import.element.as_ref() {
            let definition = module
                .types
                .iter()
                .find(|type_def| type_def.name.get() == element.get())
                .ok_or_else(|| MissingDefinition {
                    name: element.get().to_owned(),
                    path: module.path_to_string(),
                    used: element.span().into(),
                })?;

            match definition.kind {
                DeclarationKind::Alias => Err(InvalidKind {
                    kind: "alias",
                    declared: definition.name.span().into(),
                    used: element.span().into(),
                }
                .into()),
                DeclarationKind::Const => Err(InvalidKind {
                    kind: "const",
                    declared: definition.name.span().into(),
                    used: element.span().into(),
                }
                .into()),
                DeclarationKind::Struct { generics } | DeclarationKind::Enum { generics } => {
                    Ok(ResolvedImport::Type {
                        schema: self.schema,
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
            .is_some_and(|(first, _)| first.get() == self.name)
        {
            self
        } else {
            ty.path.iter().try_fold(self, |module, (name, _)| {
                module.modules.get(name.get()).ok_or_else(|| MissingModule {
                    name: name.get().to_owned(),
                    path: module.path_to_string(),
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
                path: module.path_to_string(),
                used: ty.name.span().into(),
            })?;

        match definition.kind {
            DeclarationKind::Struct { generics } | DeclarationKind::Enum { generics }
                if generics != ty.generics.as_ref().map_or(0, Punctuated::len) =>
            {
                Err(RemoteGenericsCount {
                    amount: ty.generics.as_ref().map_or(0, Punctuated::len),
                    used: ty.name.span().into(),
                    declaration: [RemoteGenericsCountDeclaration {
                        amount: generics,
                        source_code: NamedSource::new(
                            self.schema.path.as_ref().map_or_else(
                                || "<unknown>".to_owned(),
                                |p| p.display().to_string(),
                            ),
                            self.schema.source.to_owned(),
                        ),
                        used: definition.name.span().into(),
                    }],
                }
                .into())
            }
            DeclarationKind::Alias => Err(RemoteInvalidKind {
                kind: "type alias",
                used: ty.name.span().into(),
                declaration: [RemoteInvalidKindDeclaration {
                    kind: "type alias",
                    source_code: NamedSource::new(
                        self.schema
                            .path
                            .as_ref()
                            .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                        self.schema.source.to_owned(),
                    ),
                    used: definition.name.span().into(),
                }],
            }
            .into()),
            DeclarationKind::Const => Err(RemoteInvalidKind {
                kind: "constant",
                used: ty.name.span().into(),
                declaration: [RemoteInvalidKindDeclaration {
                    kind: "constant",
                    source_code: NamedSource::new(
                        self.schema
                            .path
                            .as_ref()
                            .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                        self.schema.source.to_owned(),
                    ),
                    used: definition.name.span().into(),
                }],
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
    fn is_generic(external: &ExternalType<'_>, generics: Option<&Generics<'_>>) -> bool {
        external.generics.is_none()
            && external.path.is_empty()
            && generics
                .as_ref()
                .is_some_and(|g| g.types.values().any(|gen| gen.get() == external.name.get()))
    }

    fn resolve<'a>(
        missing: &mut Vec<LocallyMissingType<'a>>,
        ty: &'a Type<'_>,
        generics: Option<&Generics<'_>>,
        module: &Module<'_>,
    ) {
        visit_externals(ty, &mut |external| {
            if !is_generic(external, generics) {
                if let Err(e) = module.resolve_local(external) {
                    missing.push(LocallyMissingType { external, error: e });
                }
            }
        });
    }

    fn resolve_fields<'a>(
        missing: &mut Vec<LocallyMissingType<'a>>,
        fields: &'a Fields<'_>,
        generics: Option<&Generics<'_>>,
        module: &Module<'_>,
    ) {
        match fields {
            Fields::Named(_, named) => {
                for field in named.values() {
                    resolve(missing, &field.ty, generics, module);
                }
            }
            Fields::Unnamed(_, unnamed) => {
                for field in unnamed.values() {
                    resolve(missing, &field.ty, generics, module);
                }
            }
            Fields::Unit => {}
        }
    }

    for def in module.definitions {
        match def {
            Definition::Struct(s) => {
                resolve_fields(missing, &s.fields, s.generics.as_ref(), module);
            }
            Definition::Enum(e) => {
                for variant in e.variants.values() {
                    resolve_fields(missing, &variant.fields, e.generics.as_ref(), module);
                }
            }
            _ => {}
        }
    }

    for module in module.modules.values() {
        resolve_module_types(module, missing);
    }
}

pub(crate) fn resolve_types<'a>(name: &'a str, schema: &'a Schema<'a>) -> Module<'a> {
    visit_module_tree(name, schema, &[], &schema.definitions)
}

/// Build up modules from the given one all the way down to all submodules.
///
/// This builds a tree structure of elements defined in each module, so they can be looked up in a
/// 2nd step to ensure all used types are actually available and correct.
fn visit_module_tree<'a>(
    name: &'a str,
    schema: &'a Schema<'a>,
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
        schema,
        path,
        imports: Vec::new(),
        types: Vec::new(),
        modules: FxHashMap::default(),
        definitions: defs,
    };

    for def in defs {
        match def {
            Definition::Module(m) => {
                module.modules.insert(
                    m.name.get(),
                    visit_module_tree(m.name.get(), schema, &module.path, &m.definitions),
                );
            }
            Definition::Struct(s) => module.types.push(Declaration {
                kind: DeclarationKind::Struct {
                    generics: s.generics.as_ref().map_or(0, |g| g.types.len()),
                },
                name: s.name.clone(),
            }),
            Definition::Enum(e) => module.types.push(Declaration {
                kind: DeclarationKind::Enum {
                    generics: e.generics.as_ref().map_or(0, |g| g.types.len()),
                },
                name: e.name.clone(),
            }),
            Definition::TypeAlias(a) => module.types.push(Declaration {
                kind: DeclarationKind::Alias,
                name: a.name.clone(),
            }),
            Definition::Const(c) => module.types.push(Declaration {
                kind: DeclarationKind::Const,
                name: c.name.clone(),
            }),
            Definition::Import(i) => module.imports.push(i),
        }
    }

    module
}

/// Walk through the type and any type arguments, calling the visitor whenever an external type is
/// encountered.
fn visit_externals<'a>(value: &'a Type<'_>, visit: &mut impl FnMut(&'a ExternalType<'_>)) {
    match &value.value {
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
        | DataType::NonZero { .. }
        | DataType::BoxString
        | DataType::BoxBytes => {}
        DataType::Vec { ty, .. }
        | DataType::HashSet { ty, .. }
        | DataType::Option { ty, .. }
        | DataType::Array { ty, .. } => {
            visit_externals(ty, visit);
        }
        DataType::HashMap { key, value, .. } => {
            visit_externals(key, visit);
            visit_externals(value, visit);
        }
        DataType::Tuple { types, .. } => {
            for ty in types.values() {
                visit_externals(ty, visit);
            }
        }
        DataType::External(ty) => {
            visit(ty);

            if let Some(generics) = &ty.generics {
                for ty in generics.values() {
                    visit_externals(ty, visit);
                }
            }
        }
    }
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
                .ok_or_else(|| MissingSchema {
                    name: root.get().to_owned(),
                    used: root.span().into(),
                })?;

            schema.resolve_import(import)
        })
        .collect()
}

pub(crate) fn resolve_type_remotely(
    ty: LocallyMissingType<'_>,
    imports: &[ResolvedImport<'_>],
) -> Result<(), ResolveError> {
    if imports.is_empty() {
        return Err(ty.error.into());
    } else if let Some((name, _)) = ty.external.path.first() {
        let module = imports.iter().find_map(|import| match import {
            ResolvedImport::Module(module) => (module.name == name.get()).then_some(module),
            ResolvedImport::Type { .. } => None,
        });

        match module {
            Some(module) => module.resolve_remote(ty.external)?,
            None => {
                return Err(ResolveRemote::MissingImport(MissingImport {
                    ty: format!(
                        "{}{}",
                        ty.external
                            .path
                            .iter()
                            .fold(String::new(), |mut acc, (part, _)| {
                                acc.push_str(part.get());
                                acc.push_str(token::DoubleColon::VALUE);
                                acc
                            }),
                        ty.external.name
                    ),
                    used: ty.external.name.span().into(),
                })
                .into());
            }
        }
    } else {
        let found = imports.iter().find_map(|import| match import {
            ResolvedImport::Module(_) => None,
            ResolvedImport::Type {
                schema,
                name,
                generics,
            } => (name.get() == ty.external.name.get()).then_some((schema, name, *generics)),
        });

        if let Some((schema, name, generics)) = found {
            if generics == ty.external.generics.as_ref().map_or(0, Punctuated::len) {
                return Ok(());
            }

            return Err(ResolveRemote::GenericsCount(RemoteGenericsCount {
                amount: ty.external.generics.as_ref().map_or(0, Punctuated::len),
                used: ty.external.name.span().into(),
                declaration: [RemoteGenericsCountDeclaration {
                    amount: generics,
                    source_code: NamedSource::new(
                        schema
                            .path
                            .as_ref()
                            .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string()),
                        schema.source.to_owned(),
                    ),
                    used: name.span().into(),
                }],
            })
            .into());
        }

        return Err(ResolveRemote::MissingImport(MissingImport {
            ty: ty.external.name.get().to_owned(),
            used: ty.external.name.span().into(),
        })
        .into());
    }

    Err(ty.error.into())
}
