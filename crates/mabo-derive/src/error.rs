use std::fmt::Write;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed, spanned::Spanned};

use crate::attributes::{FieldAttributes, StructAttributes};

macro_rules! bail {
    ($tokens:expr, $($arg:tt)*) => {
        return Err(syn::Error::new($tokens.span(), format!($($arg)*)))
    };
}

pub fn expand(derive: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &derive.ident;
    let span = derive.span();

    let Data::Struct(data) = derive.data else {
        bail!(derive, "only structs supported")
    };

    let Fields::Named(FieldsNamed { named: fields, .. }) = data.fields else {
        bail!(data.fields, "only named structs supported")
    };

    let Some(_cause_field) = fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|ident| ident == "cause"))
    else {
        bail!(fields, "struct musn't be empty");
    };

    let info = StructInfo {
        attr: StructAttributes::parse(&derive.attrs, span)?,
        fields: fields
            .iter()
            .map(|f| Ok((f, FieldAttributes::parse(&f.attrs)?)))
            .collect::<syn::Result<Vec<_>>>()?,
    };

    let error_impl = expand_error(ident, &info);
    let miette_impl = expand_miette(ident, &info);

    Ok(quote! {
        #error_impl
        #miette_impl

        impl<I> ::winnow::error::ParserError<I> for #ident
        where
            I: ::winnow::stream::Location + ::winnow::stream::Stream,
        {
            type Inner = Self;

            fn from_input(input: &I) -> Self {
                Self {
                    at: input.current_token_start()..input.current_token_start(),
                    cause: Cause::Parser(input.current_token_start()),
                }
            }

            fn into_inner(self) -> ::winnow::Result<Self::Inner, Self> {
                Ok(self)
            }
        }
    })
}

struct StructInfo<'a> {
    attr: StructAttributes,
    fields: Vec<(&'a Field, Option<FieldAttributes>)>,
}

fn expand_error(ident: &Ident, info: &StructInfo<'_>) -> TokenStream {
    let StructAttributes { msg, .. } = &info.attr;

    quote! {
        impl std::error::Error for #ident {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.cause as &(dyn std::error::Error + 'static))
            }
        }

        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(#msg)
            }
        }
    }
}

fn expand_miette(ident: &Ident, info: &StructInfo<'_>) -> TokenStream {
    let StructAttributes { code, help, .. } = &info.attr;

    let code = code.segments.iter().fold(String::new(), |mut acc, seg| {
        if !acc.is_empty() {
            acc.push_str("::");
        }
        write!(&mut acc, "{}", seg.ident).ok();
        acc
    });

    let struct_ident = info.attr.rename.as_ref().unwrap_or(ident);

    let url = {
        let item = format!("struct.{struct_ident}.html");

        quote! {
            Some(Box::new(
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
    };

    let labels = info
        .fields
        .iter()
        .enumerate()
        .filter_map(|(i, (field, info))| {
            info.as_ref().map(|info| {
                let name = field
                    .ident
                    .clone()
                    .unwrap_or_else(|| quote::format_ident!("unnamed_{i}"));
                let text = info
                    .label
                    .1
                    .as_ref()
                    .map(|s| quote! { Some(#s.to_owned()) });
                (
                    if field.ident.is_some() {
                        quote! { #name }
                    } else {
                        quote! { #i: #name }
                    },
                    quote! {
                        miette::LabeledSpan::new_with_span(#text, #name.clone())
                    },
                )
            })
        })
        .collect::<Vec<_>>();

    let labels_names = labels.iter().map(|v| &v.0);
    let labels_contents = labels.iter().map(|v| &v.1);

    quote! {
        impl miette::Diagnostic for #ident {
            fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
               Some(Box::new(#code))
            }

            fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
                Some(Box::new(format!(#help)))
            }

            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> +'_>> {
                let Self { #(#labels_names,)* .. } = self;
                Some(Box::new(vec![#(#labels_contents,)*].into_iter()))
            }

            fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn miette::Diagnostic> + '_>> {
                Some(Box::new(std::iter::once::<&dyn miette::Diagnostic>(&self.cause)))
            }

            fn url(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
                #url
            }
        }
    }
}
