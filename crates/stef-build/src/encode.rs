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
    let name = Ident::new(name.get(), Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let fields = compile_struct_fields(fields);

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Encode for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::explicit_auto_deref,
                clippy::needless_borrow,
                clippy::too_many_lines,
            )]
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
                     ..
                 }| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let name = proc_macro2::Ident::new(name.get(), Span::call_site());

                    if let DataType::Option(ty) = ty {
                        let ty = compile_data_type(ty, if is_copy(ty) {
                            quote! { *v }
                        } else {
                            quote! { v }
                        });
                        quote! { ::stef::buf::encode_field_option(w, #id, &self.#name, |w, v| { #ty; }); }
                    } else {
                        let ty = compile_data_type(ty, quote! { self.#name });
                        quote! { ::stef::buf::encode_field(w, #id, |w| { #ty; }); }
                    }
                },
            );

            quote! {
               #(#calls)*
               ::stef::buf::encode_u32(w, ::stef::buf::END_MARKER);
            }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id, .. })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    let ty = compile_data_type(ty, quote! { self.#idx });

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                });

            quote! {
               #(#calls)*
               ::stef::buf::encode_u32(w, ::stef::buf::END_MARKER);
            }
        }
        Fields::Unit => quote! {},
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
    let name = Ident::new(name.get(), Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let variants = variants.iter().map(compile_variant);

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Encode for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::semicolon_if_nothing_returned,
                clippy::too_many_lines,
            )]
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
        ..
    }: &Variant<'_>,
) -> TokenStream {
    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
    let name = Ident::new(name.get(), Span::call_site());
    let fields_body = compile_variant_fields(fields);

    match fields {
        Fields::Named(named) => {
            let field_names = named
                .iter()
                .map(|NamedField { name, .. }| Ident::new(name.get(), Span::call_site()));

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
                     ..
                 }| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let name = proc_macro2::Ident::new(name.get(), Span::call_site());

                    if matches!(ty, DataType::Option(_)) {
                        quote! { ::stef::buf::encode_field_option(w, #id, &#name); }
                    } else {
                        let ty = compile_data_type(ty, quote! { *#name });
                        quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                    }
                },
            );

            quote! {
               #(#calls)*
               ::stef::buf::encode_u32(w, ::stef::buf::END_MARKER);
            }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id, .. })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let name = Ident::new(&format!("n{idx}"), Span::call_site());
                    let ty = compile_data_type(ty, quote! { *#name });

                    quote! { ::stef::buf::encode_field(w, #id, |w| { #ty }); }
                });

            quote! {
                #(#calls)*
                ::stef::buf::encode_u32(w, ::stef::buf::END_MARKER);
            }
        }
        Fields::Unit => quote! {},
    }
}

fn compile_generics(Generics(types): &Generics<'_>) -> (TokenStream, TokenStream) {
    (!types.is_empty())
        .then(|| {
            let types = types
                .iter()
                .map(|ty| Ident::new(ty.get(), Span::call_site()));
            let types2 = types.clone();

            (
                quote! { <#(#types,)*> },
                quote! { where #(#types2: ::stef::buf::Encode,)* },
            )
        })
        .unwrap_or_default()
}

fn is_copy(ty: &DataType<'_>) -> bool {
    matches!(
        ty,
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
    )
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
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
        DataType::Vec(ty) => {
            let ty = compile_data_type(
                ty,
                if is_copy(ty) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::encode_vec(w, &#name, |w, v| { #ty; }) }
        }
        DataType::HashMap(kv) => {
            let ty_k = compile_data_type(
                &kv.0,
                if is_copy(&kv.0) {
                    quote! { *k }
                } else {
                    quote! { k }
                },
            );
            let ty_v = compile_data_type(
                &kv.1,
                if is_copy(&kv.1) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::encode_hash_map(w, &#name, |w, k| { #ty_k; }, |w, v| { #ty_v; }) }
        }
        DataType::HashSet(ty) => {
            let ty = compile_data_type(
                ty,
                if is_copy(ty) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::encode_hash_set(w, &#name, |w, v| { #ty; }) }
        }
        DataType::Option(ty) => {
            let ty = compile_data_type(
                ty,
                if is_copy(ty) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::encode_option(w, &#name, |w, v| { #ty; }) }
        }
        DataType::NonZero(ty) => match &**ty {
            DataType::U8 => quote! { ::stef::buf::encode_u8(w, #name.get()) },
            DataType::U16 => quote! { ::stef::buf::encode_u16(w, #name.get()) },
            DataType::U32 => quote! { ::stef::buf::encode_u32(w, #name.get()) },
            DataType::U64 => quote! { ::stef::buf::encode_u64(w, #name.get()) },
            DataType::U128 => quote! { ::stef::buf::encode_u128(w, #name.get()) },
            DataType::I8 => quote! { ::stef::buf::encode_i8(w, #name.get()) },
            DataType::I16 => quote! { ::stef::buf::encode_i16(w, #name.get()) },
            DataType::I32 => quote! { ::stef::buf::encode_i32(w, #name.get()) },
            DataType::I64 => quote! { ::stef::buf::encode_i64(w, #name.get()) },
            DataType::I128 => quote! { ::stef::buf::encode_i128(w, #name.get()) },
            DataType::String
            | DataType::StringRef
            | DataType::Bytes
            | DataType::BytesRef
            | DataType::Vec(_)
            | DataType::HashMap(_)
            | DataType::HashSet(_) => compile_data_type(ty, quote! { #name.get() }),
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },

        DataType::BoxString => quote! { ::stef::buf::encode_string(w, &*#name) },
        DataType::BoxBytes => quote! { ::stef::buf::encode_bytes(w, &*#name) },
        DataType::Tuple(types) => match types.len() {
            2..=12 => {
                let types = types.iter().enumerate().map(|(idx, ty)| {
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    compile_data_type(
                        ty,
                        if is_copy(ty) {
                            quote! { #name.#idx }
                        } else {
                            quote! { &(#name.#idx) }
                        },
                    )
                });
                quote! { #(#types;)* }
            }
            0 => panic!("tuple with zero elements"),
            1 => panic!("tuple with single element"),
            _ => panic!("tuple with more than 12 elements"),
        },
        DataType::Array(ty, _size) => {
            let ty = compile_data_type(
                ty,
                if is_copy(ty) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::encode_array(w, &#name, |w, v| { #ty; }) }
        }
        DataType::External(_) => {
            quote! { (#name).encode(w) }
        }
    }
}
