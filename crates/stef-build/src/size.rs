use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use stef_parser::{
    DataType, Enum, Fields, Generics, NamedField, Struct, Type, UnnamedField, Variant,
};

use crate::{BytesType, Opts};

pub(super) fn compile_struct(
    opts: &Opts,
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
    let fields = compile_struct_fields(opts, fields);

    quote! {
        #[automatically_derived]
        impl #generics ::stef::buf::Size for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::explicit_auto_deref,
                clippy::needless_borrow,
                clippy::too_many_lines,
            )]
            fn size(&self) -> usize {
                #fields
            }
        }
    }
}

fn compile_struct_fields(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
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

                    if let DataType::Option(ty) = &ty.value {
                        let ty = compile_data_type(
                            opts,
                            ty,
                            if is_copy(&ty.value) {
                                quote! { *v }
                            } else {
                                quote! { v }
                            },
                        );
                        quote! {
                            ::stef::buf::size_field_option(#id, self.#name.as_ref(), |v| { #ty })
                        }
                    } else {
                        let ty = compile_data_type(opts, ty, quote! { self.#name });
                        quote! { ::stef::buf::size_field(#id, || { #ty }) }
                    }
                },
            );

            quote! {
               #(#calls +)*
               ::stef::buf::size_u32(::stef::buf::END_MARKER)
            }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id, .. })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);

                    if let DataType::Option(ty) = &ty.value {
                        let ty = compile_data_type(
                            opts,
                            ty,
                            if is_copy(&ty.value) {
                                quote! { *v }
                            } else {
                                quote! { v }
                            },
                        );
                        quote! {
                            ::stef::buf::size_field_option(#id, self.#idx.as_ref(), |v| { #ty })
                        }
                    } else {
                        let ty = compile_data_type(opts, ty, quote! { self.#idx });
                        quote! { ::stef::buf::size_field(#id, || { #ty }) }
                    }
                });

            quote! {
               #(#calls +)*
               ::stef::buf::size_u32(::stef::buf::END_MARKER)
            }
        }
        Fields::Unit => quote! { 0 },
    }
}

pub(super) fn compile_enum(
    opts: &Opts,
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
    let variants = variants.iter().map(|v| compile_variant(opts, v));

    quote! {
        #[automatically_derived]
        impl #generics ::stef::buf::Size for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::semicolon_if_nothing_returned,
                clippy::too_many_lines,
            )]
            fn size(&self) -> usize {
                match self {
                    #(#variants,)*
                }
            }
        }
    }
}

fn compile_variant(
    opts: &Opts,
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
    let fields_body = compile_variant_fields(opts, fields);

    match fields {
        Fields::Named(named) => {
            let field_names = named
                .iter()
                .map(|NamedField { name, .. }| Ident::new(name.get(), Span::call_site()));

            quote! {
                Self::#name{ #(#field_names,)* } => {
                    ::stef::buf::size_id(#id) +
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
                    ::stef::buf::size_id(#id) +
                    #fields_body
                }
            }
        }
        Fields::Unit => quote! {
            Self::#name => {
                ::stef::buf::size_id(#id)
            }
        },
    }
}

fn compile_variant_fields(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
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

                    if matches!(ty.value, DataType::Option(_)) {
                        quote! { ::stef::buf::size_field_option(#id, #name.as_ref()) }
                    } else {
                        let ty = compile_data_type(opts, ty, quote! { *#name });
                        quote! { ::stef::buf::size_field(#id, || { #ty }) }
                    }
                },
            );

            quote! {
               #(#calls +)*
               ::stef::buf::size_u32(::stef::buf::END_MARKER)
            }
        }
        Fields::Unnamed(unnamed) => {
            let calls = unnamed
                .iter()
                .enumerate()
                .map(|(idx, UnnamedField { ty, id, .. })| {
                    let id = proc_macro2::Literal::u32_unsuffixed(id.get());
                    let name = Ident::new(&format!("n{idx}"), Span::call_site());

                    if matches!(ty.value, DataType::Option(_)) {
                        quote! { ::stef::buf::size_field_option(#id, #name.as_ref()) }
                    } else {
                        let ty = compile_data_type(opts, ty, quote! { *#name });
                        quote! { ::stef::buf::size_field(#id, || { #ty }) }
                    }
                });

            quote! {
                #(#calls +)*
                ::stef::buf::size_u32(::stef::buf::END_MARKER)
            }
        }
        Fields::Unit => quote! { 0 },
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
                quote! { where #(#types2: ::stef::buf::Size,)* },
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
fn compile_data_type(opts: &Opts, ty: &Type<'_>, name: TokenStream) -> TokenStream {
    match &ty.value {
        DataType::Bool => quote! { ::stef::buf::size_bool(#name) },
        DataType::U8 => quote! { ::stef::buf::size_u8(#name) },
        DataType::U16 => quote! { ::stef::buf::size_u16(#name) },
        DataType::U32 => quote! { ::stef::buf::size_u32(#name) },
        DataType::U64 => quote! { ::stef::buf::size_u64(#name) },
        DataType::U128 => quote! { ::stef::buf::size_u128(#name) },
        DataType::I8 => quote! { ::stef::buf::size_i8(#name) },
        DataType::I16 => quote! { ::stef::buf::size_i16(#name) },
        DataType::I32 => quote! { ::stef::buf::size_i32(#name) },
        DataType::I64 => quote! { ::stef::buf::size_i64(#name) },
        DataType::I128 => quote! { ::stef::buf::size_i128(#name) },
        DataType::F32 => quote! { ::stef::buf::size_f32(#name) },
        DataType::F64 => quote! { ::stef::buf::size_f64(#name) },
        DataType::String | DataType::StringRef | DataType::BoxString => {
            quote! { ::stef::buf::size_string(&#name) }
        }
        DataType::Bytes | DataType::BytesRef | DataType::BoxBytes => match opts.bytes_type {
            BytesType::VecU8 => quote! { ::stef::buf::size_bytes_std(&#name) },
            BytesType::Bytes => quote! { ::stef::buf::size_bytes_bytes(&#name) },
        },
        DataType::Vec(ty) => {
            let ty = compile_data_type(
                opts,
                ty,
                if is_copy(&ty.value) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::size_vec(&#name, |v| { #ty }) }
        }
        DataType::HashMap(kv) => {
            let ty_k = compile_data_type(
                opts,
                &kv.0,
                if is_copy(&kv.0.value) {
                    quote! { *k }
                } else {
                    quote! { k }
                },
            );
            let ty_v = compile_data_type(
                opts,
                &kv.1,
                if is_copy(&kv.1.value) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::size_hash_map(&#name, |k| { #ty_k }, |v| { #ty_v }) }
        }
        DataType::HashSet(ty) => {
            let ty = compile_data_type(
                opts,
                ty,
                if is_copy(&ty.value) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::size_hash_set(&#name, |v| { #ty }) }
        }
        DataType::Option(ty) => {
            let ty = compile_data_type(
                opts,
                ty,
                if is_copy(&ty.value) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::size_option(#name.as_ref(), |v| { #ty }) }
        }
        DataType::NonZero(ty) => match &ty.value {
            DataType::U8 => quote! { ::stef::buf::size_u8(#name.get()) },
            DataType::U16 => quote! { ::stef::buf::size_u16(#name.get()) },
            DataType::U32 => quote! { ::stef::buf::size_u32(#name.get()) },
            DataType::U64 => quote! { ::stef::buf::size_u64(#name.get()) },
            DataType::U128 => quote! { ::stef::buf::size_u128(#name.get()) },
            DataType::I8 => quote! { ::stef::buf::size_i8(#name.get()) },
            DataType::I16 => quote! { ::stef::buf::size_i16(#name.get()) },
            DataType::I32 => quote! { ::stef::buf::size_i32(#name.get()) },
            DataType::I64 => quote! { ::stef::buf::size_i64(#name.get()) },
            DataType::I128 => quote! { ::stef::buf::size_i128(#name.get()) },
            DataType::String
            | DataType::StringRef
            | DataType::Bytes
            | DataType::BytesRef
            | DataType::Vec(_)
            | DataType::HashMap(_)
            | DataType::HashSet(_) => compile_data_type(opts, ty, quote! { #name.get() }),
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        DataType::Tuple(types) => match types.len() {
            2..=12 => {
                let types = types.iter().enumerate().map(|(idx, ty)| {
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    compile_data_type(
                        opts,
                        ty,
                        if is_copy(&ty.value) {
                            quote! { #name.#idx }
                        } else {
                            quote! { &(#name.#idx) }
                        },
                    )
                });
                quote! { #(#types)+* }
            }
            n => todo!("compiler should catch invalid tuple with {n} elements"),
        },
        DataType::Array(ty, _size) => {
            let ty = compile_data_type(
                opts,
                ty,
                if is_copy(&ty.value) {
                    quote! { *v }
                } else {
                    quote! { v }
                },
            );
            quote! { ::stef::buf::size_array(&#name, |v| { #ty }) }
        }
        DataType::External(_) => {
            quote! { (#name).size() }
        }
    }
}
