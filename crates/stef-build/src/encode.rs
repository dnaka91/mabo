use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use stef_parser::{DataType, Enum, Fields, NamedField, Struct, UnnamedField, Variant};

pub fn compile_struct(
    Struct {
        comment: _,
        attributes: _,
        name,
        generics: _,
        fields,
    }: &Struct<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let fields = compile_struct_fields(fields);

    quote! {
        impl ::stef::Encode for #name {
            fn encode(&self, w: &mut impl ::stef::BufMut) {
                #fields
            }
        }
    }
}

fn compile_struct_fields(fields: &Fields<'_>) -> TokenStream {
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
                    let ty = compile_data_type(ty, quote! { self.#name });

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                },
            );

            quote! { #(#calls)* }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.0);
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    let ty = compile_data_type(ty, quote! { self.#idx });

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                });

            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

pub fn compile_enum(
    Enum {
        comment: _,
        attributes: _,
        name,
        generics: _,
        variants,
    }: &Enum<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let variants = variants.iter().map(compile_variant);

    quote! {
        impl ::stef::Encode for #name {
            fn encode(&self, w: &mut impl ::stef::BufMut) {
                match self {
                    #(#variants,)*
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
    let fields_body = compile_variant_fields(fields);

    match fields {
        Fields::Named(named) => {
            let field_names = named
                .iter()
                .map(|NamedField { name, .. }| Ident::new(name, Span::call_site()));

            quote! {
                Self::#name{ #(#field_names,)* } => {
                    ::stef::buf::encode_id(w, #id);
                    #fields_body
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let field_names = unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| Ident::new(&format!("n{idx}"), Span::call_site()));

            quote! {
                Self::#name(#(#field_names,)*) => {
                    ::stef::buf::encode_id(w, #id);
                    #fields_body
                }
            }
        }
        Fields::Unit => quote! {
            Self::#name => {
                ::stef::buf::encode_id(w, #id);
                #fields_body
            }
        },
    }
}

fn compile_variant_fields(fields: &Fields<'_>) -> TokenStream {
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
                    let ty = compile_data_type(ty, quote! { #name });

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                },
            );

            quote! { #(#calls)* }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.0);
                    let name = Ident::new(&format!("n{idx}"), Span::call_site());
                    let ty = compile_data_type(ty, name.to_token_stream());

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                });

            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

#[allow(clippy::needless_pass_by_value)]
fn compile_data_type(ty: &DataType<'_>, name: TokenStream) -> TokenStream {
    match ty {
        DataType::Bool => quote! { ::stef::buf::encode_bool(w, #name) },
        DataType::U8 => quote! { ::stef::buf::encode_u8(w, #name) },
        DataType::U16 => quote! { ::stef::buf::encode_u16(w, #name) },
        DataType::U32 => quote! { ::stef::buf::encode_u32(w, #name) },
        DataType::U64 => quote! { ::stef::buf::encode_u64(w, #name) },
        DataType::U128 => quote! { ::stef::buf::encode_u128(w, #name) },
        DataType::I8 => quote! { ::stef::buf::encode_i8(w, #name) },
        DataType::I16 => quote! { ::stef::buf::encode_i16(w, #name) },
        DataType::I32 => quote! { ::stef::buf::encode_i32(w, #name) },
        DataType::I64 => quote! { ::stef::buf::encode_i64(w, #name) },
        DataType::I128 => quote! { ::stef::buf::encode_i128(w, #name) },
        DataType::F32 => quote! { ::stef::buf::encode_f32(w, #name) },
        DataType::F64 => quote! { ::stef::buf::encode_f64(w, #name) },
        DataType::String | DataType::StringRef => quote! { ::stef::buf::encode_string(w, &#name) },
        DataType::Bytes | DataType::BytesRef => quote! { ::stef::buf::encode_bytes(w, &#name) },
        DataType::Vec(_ty) => quote! { ::stef::buf::encode_vec(w, &#name) },
        DataType::HashMap(_kv) => quote! { ::stef::buf::encode_hash_map(w, #name) },
        DataType::HashSet(_ty) => quote! { ::stef::buf::encode_hash_set(w, #name) },
        DataType::Option(_ty) => quote! { ::stef::buf::encode_option(w, #name) },
        DataType::BoxString => quote! { ::stef::buf::encode_string(w, &*#name) },
        DataType::BoxBytes => quote! { ::stef::buf::encode_bytes(w, &*#name) },
        DataType::Tuple(types) => match types.len() {
            size @ 2..=12 => {
                let fn_name = Ident::new(&format!("encode_tuple{size}"), Span::call_site());
                quote! { ::stef::buf::#fn_name(w, &#name) }
            }
            0 => panic!("tuple with zero elements"),
            1 => panic!("tuple with single element"),
            _ => panic!("tuple with more than 12 elements"),
        },
        DataType::Array(_ty, _size) => {
            quote! { ::stef::buf::encode_array(w, &#name) }
        }
        DataType::NonZero(_) | DataType::External(_) => {
            quote! { #name.encode(w) }
        }
    }
}
