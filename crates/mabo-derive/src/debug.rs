use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Field, Fields, Ident, Type, TypePath};

macro_rules! bail {
    ($tokens:expr, $($arg:tt)*) => {
        return Err(syn::Error::new($tokens.span(), format!($($arg)*)))
    };
}

pub fn expand(derive: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &derive.ident;
    let generics = &derive.generics;
    let name = Literal::string(&ident.to_string());

    match derive.data {
        Data::Struct(data) => {
            let fields = expand_fields(&name, &data.fields);

            Ok(quote! {
                impl #generics ::core::fmt::Debug for #ident #generics {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        match self {
                            Self #fields
                        }
                    }
                }
            })
        }
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|variant| {
                let ident = &variant.ident;
                let fields = expand_fields(&Literal::string(&ident.to_string()), &variant.fields);

                quote! {
                    Self::#ident #fields
                }
            });

            Ok(quote! {
                impl #generics ::core::fmt::Debug for #ident #generics {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        match self {
                            #(#variants,)*
                        }
                    }
                }
            })
        }
        Data::Union(_) => bail!(derive, "unions not supported"),
    }
}

fn expand_fields(name: &Literal, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                if filter_span(&field.ty) {
                    quote! { #ident: _ }
                } else {
                    quote! { #ident }
                }
            });

            let fields = fields
                .named
                .iter()
                .filter(|&field| !filter_span(&field.ty))
                .map(expand_named_field);

            quote! {
                {#(#names,)*} => f.debug_struct(#name)
                    #(#fields)*
                    .finish()
            }
        }
        Fields::Unnamed(fields) => {
            let names = fields.unnamed.iter().enumerate().map(|(i, field)| {
                if filter_span(&field.ty) {
                    quote! { _ }
                } else {
                    let ident = Ident::new(&format!("n{i}"), Span::call_site());
                    quote! { #ident }
                }
            });

            let fields = fields
                .unnamed
                .iter()
                .enumerate()
                .filter(|(_, field)| !filter_span(&field.ty))
                .map(expand_unnamed_field);

            quote! {
                (#(#names,)*) => f.debug_tuple(#name)
                    #(#fields)*
                    .finish()
            }
        }
        Fields::Unit => quote! { => f.write_str(#name) },
    }
}

fn expand_named_field(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let name = Literal::string(&ident.to_string());

    quote! {
        .field(#name, &#ident)
    }
}

fn expand_unnamed_field((i, field): (usize, &Field)) -> TokenStream {
    if let Type::Tuple(tuple) = &field.ty {
        let ident = Ident::new(&format!("n{i}"), Span::call_site());
        let index = tuple
            .elems
            .iter()
            .enumerate()
            .filter(|(_, ty)| !filter_span(ty))
            .map(|(j, _)| Literal::usize_unsuffixed(j));

        quote! {
            .field(&#(#ident.#index,)*)
        }
    } else {
        let ident = Ident::new(&format!("n{i}"), Span::call_site());

        quote! {
            .field(&#ident)
        }
    }
}

fn filter_span(ty: &Type) -> bool {
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };

    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "Span")
}
