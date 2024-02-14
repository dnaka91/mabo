use std::fmt::Write;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    spanned::Spanned, Data, DeriveInput, Field, Fields, GenericArgument, Index, PathArguments,
    Type, TypePath, Variant,
};

use crate::attributes::{EnumAttributes, FieldAttributes, VariantAttributes};

macro_rules! bail {
    ($tokens:expr, $($arg:tt)*) => {
        return Err(syn::Error::new($tokens.span(), format!($($arg)*)))
    };
}

pub fn expand(derive: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &derive.ident;
    let span = derive.span();

    let Data::Enum(data) = derive.data else {
        bail!(derive, "only enums supported")
    };

    let Some(first) = data.variants.first() else {
        bail!(data.variants, "enum musn't be empty");
    };

    is_parser_variant(first)?;

    let attrs = EnumAttributes::parse(&derive.attrs, span)?;

    let variants = data
        .variants
        .iter()
        .skip(1)
        .map(VariantInfo::parse)
        .collect::<syn::Result<Vec<_>>>()?;

    let error_impl = expand_error(ident, &variants);
    let miette_impl = expand_miette(ident, &attrs, &variants);
    let winnow_impl = expand_winnow(ident, &variants)?;

    Ok(quote! {
        #error_impl
        #miette_impl
        #winnow_impl
    })
}

fn is_parser_variant(variant: &Variant) -> syn::Result<()> {
    if variant.ident != "Parser" {
        bail!(variant, "first variant must be named `Parser`");
    }

    let Fields::Unnamed(fields) = &variant.fields else {
        bail!(variant, "first variant must be unnamed");
    };

    if fields.unnamed.len() != 2 {
        bail!(
            fields,
            "first variant must contain exactly two unnamed fields"
        );
    };

    let Type::Path(ty) = &fields.unnamed[0].ty else {
        bail!(fields.unnamed[0], "first variant type invalid");
    };

    if !compare_path(ty, &[&["ErrorKind"], &["winnow", "error", "ErrorKind"]]) {
        bail!(ty, "first variant type must be `ErrorKind`");
    }

    let Type::Path(ty) = &fields.unnamed[1].ty else {
        bail!(fields.unnamed[1], "second variant type invalid");
    };

    if !compare_path(ty, &[&["usize"]]) {
        bail!(ty, "second variant type must be `usize`");
    }

    Ok(())
}

fn compare_path(ty: &TypePath, paths: &[&[&str]]) -> bool {
    ty.qself.is_none()
        && paths.iter().any(|&path| {
            ty.path.segments.len() == path.len()
                && ty
                    .path
                    .segments
                    .iter()
                    .zip(path)
                    .all(|(a, b)| a.arguments.is_none() && a.ident == b)
        })
}

struct VariantInfo<'a> {
    variant: &'a Variant,
    attr: VariantAttributes,
    fields: Vec<(&'a Field, Option<FieldAttributes>)>,
}

impl<'a> VariantInfo<'a> {
    fn parse(variant: &'a Variant) -> syn::Result<Self> {
        Ok(Self {
            variant,
            attr: VariantAttributes::parse(&variant.attrs, variant.span())?,
            fields: match &variant.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f| Ok((f, FieldAttributes::parse(&f.attrs)?)))
                    .collect::<syn::Result<Vec<_>>>()?,
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .map(|f| Ok((f, FieldAttributes::parse(&f.attrs)?)))
                    .collect::<syn::Result<Vec<_>>>()?,
                Fields::Unit => Vec::new(),
            },
        })
    }
}

fn expand_error(ident: &Ident, variants: &[VariantInfo<'_>]) -> TokenStream {
    let sources = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        match &v.attr {
            VariantAttributes::Error { .. } => quote! {
                Self::#ident { .. } => None
            },
            VariantAttributes::External => {
                if v.fields.len() == 1 {
                    quote! {
                        Self::#ident(inner) => Some(inner)
                    }
                } else {
                    quote! {
                        Self::#ident { cause, .. } => Some(cause)
                    }
                }
            }
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.source()
            },
        }
    });

    let fmts = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        match &v.attr {
            VariantAttributes::Error { msg, .. } => quote! {
                Self::#ident { .. } => f.write_str(#msg)
            },
            VariantAttributes::External => {
                if v.fields.len() == 1 {
                    quote! {
                        Self::#ident(inner) => inner.fmt(f)
                    }
                } else {
                    quote! {
                        Self::#ident { cause, ..} => cause.fmt(f)
                    }
                }
            }
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.fmt(f)
            },
        }
    });

    let froms = variants
        .iter()
        .filter(|v| matches!(v.attr, VariantAttributes::Forward))
        .map(|v| {
            let variant_ident = &v.variant.ident;
            let Fields::Unnamed(fields) = &v.variant.fields else {
                panic!("expected unnamed fields")
            };
            let ty = &fields.unnamed[0].ty;

            let boxed_type = match ty {
                Type::Path(ty) => ty.path.segments.first().and_then(|seg| {
                    if seg.ident != "Box" {
                        return None;
                    }

                    match &seg.arguments {
                        PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
                            match &args.args[0] {
                                GenericArgument::Type(ty) => Some(ty),
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }),
                _ => None,
            };

            let from_boxed = boxed_type.map(|ty| {
                quote! {
                    impl From<#ty> for #ident {
                        fn from(source: #ty) -> Self {
                            Self::#variant_ident(Box::new(source))
                        }
                    }
                }
            });

            quote! {
                impl From<#ty> for #ident {
                    fn from(source: #ty) -> Self {
                        Self::#variant_ident(source)
                    }
                }

                #from_boxed
            }
        });

    quote! {
        impl std::error::Error for #ident {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#sources,)*
                }
            }
        }

        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Parser(kind, _) => kind.fmt(f),
                    #(#fmts,)*
                }
            }
        }

        #(#froms)*
    }
}

fn expand_miette(
    ident: &Ident,
    attrs: &EnumAttributes,
    variants: &[VariantInfo<'_>],
) -> TokenStream {
    let codes = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        match &v.attr {
            VariantAttributes::Error { code, .. } => {
                let code = code.segments.iter().fold(String::new(), |mut acc, seg| {
                    if !acc.is_empty() {
                        acc.push_str("::");
                    }
                    write!(&mut acc, "{}", seg.ident).ok();
                    acc
                });

                quote! {
                    Self::#ident { .. } => Some(Box::new(#code))
                }
            }
            VariantAttributes::External => quote! {
                Self::#ident { .. } => None
            },
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.code()
            },
        }
    });

    let helps = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        let fields = v.fields.iter().enumerate().map(|(i, (field, _))| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| quote::format_ident!("_{i}"))
        });

        match &v.attr {
            VariantAttributes::Error { help, .. } => quote! {
                #[allow(unused_variables)]
                Self::#ident { #(#fields,)* } => Some(Box::new(format!(#help)))
            },
            VariantAttributes::External => quote! {
                Self::#ident { .. } => None
            },
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.help()
            },
        }
    });

    let labels = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        match &v.attr {
            VariantAttributes::Error {.. } => {
                let fields = v
                    .fields
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (f, info))| info.as_ref().map(|info| (i, f, info)))
                    .filter(|(_, _, info)| info.label.0)
                    .map(|(i, field, info)| {
                        let idx = Index::from(i);
                        let name = field
                            .ident
                            .clone()
                            .unwrap_or_else(|| quote::format_ident!("_{i}"));
                        let text = info.label.1.as_ref().map(|s| quote!{ Some(#s.to_owned()) });
                        (
                            if field.ident.is_some() {
                                quote! { #name }
                            } else {
                                quote! { #idx: #name }
                            },
                            quote! {
                                miette::LabeledSpan::new_with_span(#text, #name.clone())
                            },
                        )
                    })
                    .collect::<Vec<_>>();

                let fields_names = fields.iter().map(|(name, _)| name);
                let fields_contents = fields.iter().map(|(_, content)| content);

                quote! {
                    #[allow(unused_variables)]
                    Self::#ident { #(#fields_names,)* .. } => Some(Box::new(vec![#(#fields_contents,)*].into_iter()))
                }
            }
            VariantAttributes::External => quote! {
                Self::#ident { .. } => None
            },
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.labels()
            },
        }
    });

    let relateds = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        if matches!(v.attr, VariantAttributes::Forward) {
            quote! {
                Self::#ident(inner) => inner.related()
            }
        } else {
            quote! {
                Self::#ident { .. } => None
            }
        }
    });

    let enum_ident = attrs.rename.as_ref().unwrap_or(ident);

    let urls = variants.iter().map(|v| {
        let ident = &v.variant.ident;

        match &v.attr {
            VariantAttributes::Error { .. } => {
                let item = format!("enum.{enum_ident}.html#variant.{ident}");

                quote! {
                    Self::#ident { .. } => Some(Box::new(
                        concat!(
                            "https://docs.rs/",
                            env!("CARGO_PKG_NAME"), "/",
                            env!("CARGO_PKG_VERSION"), "/",
                            env!("CARGO_CRATE_NAME"),
                            "/error/",
                            #item,
                        )
                    ))
                }
            }
            VariantAttributes::External => quote! {
                Self::#ident { .. } => None
            },
            VariantAttributes::Forward => quote! {
                Self::#ident(inner) => inner.url()
            },
        }
    });

    quote! {
        impl miette::Diagnostic for #ident {
            fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#codes,)*
                }
            }

            fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#helps,)*
                }
            }

            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> +'_>> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#labels,)*
                }
            }

            fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn miette::Diagnostic> + '_>> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#relateds,)*
                }
            }

            fn url(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
                match self {
                    Self::Parser(_, _) => None,
                    #(#urls,)*
                }
            }
        }
    }
}

fn expand_winnow(ident: &Ident, variants: &[VariantInfo<'_>]) -> syn::Result<TokenStream> {
    let externals = variants.iter()
    .filter(|v| matches!(v.attr, VariantAttributes::External))
    .map(|v| {
        let variant_ident = &v.variant.ident;

        match v.fields.len() {
            1 => {
                let ty = &v.fields[0].0.ty;

                Ok(quote! {
                    impl<I> ::winnow::error::FromExternalError<I, #ty> for #ident {
                        fn from_external_error(_: &I, _: ::winnow::error::ErrorKind, e: #ty) -> Self {
                            Self::#variant_ident(e)
                        }
                    }
                })
            }
            2 => {
                let ty = &v.fields[1].0.ty;

                Ok(quote! {
                    impl<I> ::winnow::error::FromExternalError<I, #ty> for #ident
                    where
                        I: ::winnow::stream::Location,
                    {
                        fn from_external_error(input: &I, _: ::winnow::error::ErrorKind, e: #ty) -> Self {
                            Self::#variant_ident {
                                at: input.location(),
                                cause: e,
                            }
                        }
                    }
                })
            }
            _ => bail!(v.variant, "external variants must have 1 or 2 fields only"),
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl<I> ::winnow::error::ParserError<I> for #ident
        where
            I: ::winnow::stream::Location + ::winnow::stream::Stream,
        {
            fn from_error_kind(input: &I, kind: ::winnow::error::ErrorKind) -> Self {
                Self::Parser(kind, input.location())
            }

            fn append(self, _: &I, _: &I::Checkpoint, _: ::winnow::error::ErrorKind) -> Self {
                self
            }
        }

        #(#externals)*
    })
}
