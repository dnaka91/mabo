use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use stef_parser::{DataType, Enum, Fields, Generics, NamedField, Struct, UnnamedField, Variant};

pub fn compile_struct(
    Struct {
        comment: _,
        attributes: _,
        name,
        generics,
        fields,
    }: &Struct<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let field_vars = compile_field_vars(fields);
    let field_matches = compile_field_matches(fields);
    let field_assigns = compile_field_assigns(fields);

    let body = if matches!(fields, Fields::Unit) {
        quote! { Ok(Self) }
    } else {
        quote! {
            #field_vars

            loop {
                match ::stef::buf::decode_id(r)? {
                    ::stef::buf::END_MARKER => break,
                    #field_matches
                    _ => continue,
                }
            }

            Ok(Self #field_assigns)
        }
    };

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Decode for #name #generics #generics_where {
            fn decode(r: &mut impl ::stef::Buf) -> ::stef::buf::Result<Self> {
                #body
            }
        }
    }
}

pub fn compile_enum(
    Enum {
        comment: _,
        attributes: _,
        name,
        generics,
        variants,
    }: &Enum<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let variants = variants.iter().map(compile_variant);

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Decode for #name #generics #generics_where {
            fn decode(r: &mut impl ::stef::Buf) -> ::stef::buf::Result<Self> {
                match ::stef::buf::decode_id(r)? {
                    #(#variants,)*
                    id => Err(::stef::buf::Error::UnknownVariant(id)),
                }
            }
        }
    }
}

fn compile_variant(
    Variant {
        comment: _,
        name,
        fields,
        id,
    }: &Variant<'_>,
) -> TokenStream {
    let id = proc_macro2::Literal::u32_unsuffixed(id.0);
    let name = Ident::new(name, Span::call_site());
    let field_vars = compile_field_vars(fields);
    let field_matches = compile_field_matches(fields);
    let field_assigns = compile_field_assigns(fields);

    if matches!(fields, Fields::Unit) {
        quote! { #id => Ok(Self::#name) }
    } else {
        quote! {
            #id => {
                #field_vars

                loop {
                    match ::stef::buf::decode_id(r)? {
                        ::stef::buf::END_MARKER => break,
                        #field_matches
                        _ => continue,
                    }
                }

                Ok(Self::#name #field_assigns)
            }
        }
    }
}

fn compile_field_vars(fields: &Fields<'_>) -> TokenStream {
    let vars: Box<dyn Iterator<Item = _>> = match fields {
        Fields::Named(named) => Box::new(named.iter().map(|named| {
            let name = Ident::new(named.name, Span::call_site());
            (name, &named.ty)
        })),
        Fields::Unnamed(unnamed) => Box::new(unnamed.iter().enumerate().map(|(idx, unnamed)| {
            let name = Ident::new(&format!("n{idx}"), Span::call_site());
            (name, &unnamed.ty)
        })),
        Fields::Unit => return quote! {},
    };

    let vars = vars.map(|(name, ty)| {
        let ty_ident = super::definition::compile_data_type(ty);

        if matches!(ty, DataType::Option(_)) {
            quote! { let mut #name: #ty_ident = None; }
        } else {
            quote! { let mut #name: Option<#ty_ident> = None; }
        }
    });

    quote! { #(#vars)* }
}

fn compile_field_matches(fields: &Fields<'_>) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let calls = named.iter().map(
                |NamedField {
                     comment: _,
                     name,
                     ty,
                     id,
                 }| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.0);
                    let name = proc_macro2::Ident::new(name, Span::call_site());
                    let ty = compile_data_type(ty);

                    quote! { #id => #name = Some(#ty?) }
                },
            );

            quote! { #(#calls,)* }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.0);
                    let name = Ident::new(&format!("n{idx}"), Span::call_site());
                    let ty = compile_data_type(ty);

                    quote! { #id => #name = Some(#ty?) }
                });

            quote! { #(#calls,)* }
        }
        Fields::Unit => quote! {},
    }
}

fn compile_field_assigns(fields: &Fields<'_>) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let assigns = named.iter().map(|named| {
                let name = Ident::new(named.name, Span::call_site());
                let name_lit = proc_macro2::Literal::string(named.name);
                let id = proc_macro2::Literal::u32_unsuffixed(named.id.0);

                if matches!(named.ty, DataType::Option(_)) {
                    quote! { #name }
                } else {
                    quote! { #name: #name.ok_or(::stef::buf::Error::MissingField {
                        id: #id,
                        name: Some(#name_lit),
                    })? }
                }
            });

            quote! { {#(#assigns,)*} }
        }
        Fields::Unnamed(unnamed) => {
            let assigns = unnamed.iter().enumerate().map(|(idx, unnamed)| {
                let name = Ident::new(&format!("n{idx}"), Span::call_site());
                let id = proc_macro2::Literal::u32_unsuffixed(unnamed.id.0);

                if matches!(unnamed.ty, DataType::Option(_)) {
                    quote! { #name }
                } else {
                    quote! { #name.ok_or(::stef::buf::Error::MissingField {
                        id: #id,
                        name: None,
                    })? }
                }
            });

            quote! { (#(#assigns,)*) }
        }
        Fields::Unit => quote! {},
    }
}

fn compile_generics(Generics(types): &Generics<'_>) -> (TokenStream, TokenStream) {
    (!types.is_empty())
        .then(|| {
            let types = types.iter().map(|ty| Ident::new(ty, Span::call_site()));
            let types2 = types.clone();

            (
                quote! { <#(#types,)*> },
                quote! { where #(#types2: ::std::fmt::Debug + ::stef::buf::Decode,)* },
            )
        })
        .unwrap_or_default()
}

#[allow(clippy::needless_pass_by_value)]
fn compile_data_type(ty: &DataType<'_>) -> TokenStream {
    match ty {
        DataType::Bool => quote! { ::stef::buf::decode_bool(r) },
        DataType::U8 => quote! { ::stef::buf::decode_u8(r) },
        DataType::U16 => quote! { ::stef::buf::decode_u16(r) },
        DataType::U32 => quote! { ::stef::buf::decode_u32(r) },
        DataType::U64 => quote! { ::stef::buf::decode_u64(r) },
        DataType::U128 => quote! { ::stef::buf::decode_u128(r) },
        DataType::I8 => quote! { ::stef::buf::decode_i8(r) },
        DataType::I16 => quote! { ::stef::buf::decode_i16(r) },
        DataType::I32 => quote! { ::stef::buf::decode_i32(r) },
        DataType::I64 => quote! { ::stef::buf::decode_i64(r) },
        DataType::I128 => quote! { ::stef::buf::decode_i128(r) },
        DataType::F32 => quote! { ::stef::buf::decode_f32(r) },
        DataType::F64 => quote! { ::stef::buf::decode_f64(r) },
        DataType::String | DataType::StringRef => quote! { ::stef::buf::decode_string(r) },
        DataType::Bytes | DataType::BytesRef => quote! { ::stef::buf::decode_bytes(r) },
        DataType::Vec(_ty) => quote! { ::stef::buf::decode_vec(r) },
        DataType::HashMap(_kv) => quote! { ::stef::buf::decode_hash_map(r) },
        DataType::HashSet(_ty) => quote! { ::stef::buf::decode_hash_set(r) },
        DataType::Option(ty) => compile_data_type(ty),
        DataType::NonZero(ty) => match **ty {
            DataType::U8 => quote! { NonZeroU8::decode(r) },
            DataType::U16 => quote! { NonZeroU16::decode(r) },
            DataType::U32 => quote! { NonZeroU32::decode(r) },
            DataType::U64 => quote! { NonZeroU64::decode(r) },
            DataType::U128 => quote! { NonZeroU128::decode(r) },
            DataType::I8 => quote! { NonZeroI8::decode(r) },
            DataType::I16 => quote! { NonZeroI16::decode(r) },
            DataType::I32 => quote! { NonZeroI32::decode(r) },
            DataType::I64 => quote! { NonZeroI64::decode(r) },
            DataType::I128 => quote! { NonZeroI128::decode(r) },
            _ => todo!(),
        },
        DataType::BoxString => quote! { Box<str>::decode(r) },
        DataType::BoxBytes => quote! { Box<[u8]>::decode(r) },
        DataType::Tuple(types) => match types.len() {
            size @ 2..=12 => {
                let fn_name = Ident::new(&format!("decode_tuple{size}"), Span::call_site());
                quote! { ::stef::buf::#fn_name(r) }
            }
            0 => panic!("tuple with zero elements"),
            1 => panic!("tuple with single element"),
            _ => panic!("tuple with more than 12 elements"),
        },
        DataType::Array(_ty, _size) => {
            quote! { ::stef::buf::decode_array(r) }
        }
        DataType::External(ty) => {
            let ty = Ident::new(ty.name, Span::call_site());
            quote! { #ty::decode(r) }
        }
    }
}
