use mabo_compiler::simplify::{Enum, Field, FieldKind, Fields, Struct, Type, Variant};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{BytesType, Opts};

pub(super) fn compile_struct(
    opts: &Opts,
    Struct {
        name,
        generics,
        fields,
        ..
    }: &Struct<'_>,
) -> TokenStream {
    let names = fields
        .fields
        .iter()
        .map(|field| Ident::new(&field.name, Span::call_site()));
    let names = match fields.kind {
        FieldKind::Named => quote! { {#(#names,)*} },
        FieldKind::Unnamed => quote! { (#(#names,)*) },
        FieldKind::Unit => quote! {},
    };

    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let fields = compile_struct_fields(opts, fields);

    quote! {
        #[automatically_derived]
        impl #generics ::mabo::buf::Size for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::deref_addrof,
                clippy::explicit_auto_deref,
                clippy::needless_borrow,
                clippy::too_many_lines,
            )]
            fn size(&self) -> usize {
                let Self #names = self;
                #fields
            }
        }
    }
}

fn compile_struct_fields(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
    if fields.kind == FieldKind::Unit {
        quote! { 0 }
    } else {
        let calls = fields.fields.iter().map(|Field { name, ty, id, .. }| {
            let id = proc_macro2::Literal::u32_unsuffixed(*id);
            let name = proc_macro2::Ident::new(name, Span::call_site());

            if let Type::Option(ty) = &ty {
                let ty = compile_data_type(opts, ty, quote! { v });
                quote! {
                    ::mabo::buf::size_field_option(#id, #name.as_ref(), |v| { #ty })
                }
            } else {
                let ty = compile_data_type(opts, ty, name.into_token_stream());
                quote! { ::mabo::buf::size_field(#id, || { #ty }) }
            }
        });

        quote! {
            #(#calls +)*
            ::mabo::buf::END_MARKER_SIZE
        }
    }
}

pub(super) fn compile_enum(
    opts: &Opts,
    Enum {
        name,
        generics,
        variants,
        ..
    }: &Enum<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let variants = variants.iter().map(|v| compile_variant(opts, v));

    quote! {
        #[automatically_derived]
        impl #generics ::mabo::buf::Size for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::deref_addrof,
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
    let id = proc_macro2::Literal::u32_unsuffixed(*id);
    let name = Ident::new(name, Span::call_site());
    let fields_body = compile_variant_fields(opts, fields);
    let field_names = fields
        .fields
        .iter()
        .map(|field| Ident::new(&field.name, Span::call_site()));

    match fields.kind {
        FieldKind::Named => quote! {
            Self::#name{ #(#field_names,)* } => {
                ::mabo::buf::size_variant_id(#id) +
                #fields_body
            }
        },
        FieldKind::Unnamed => quote! {
            Self::#name(#(#field_names,)*) => {
                ::mabo::buf::size_variant_id(#id) +
                #fields_body
            }
        },
        FieldKind::Unit => quote! {
            Self::#name => {
                ::mabo::buf::size_variant_id(#id)
            }
        },
    }
}

fn compile_variant_fields(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
    if fields.kind == FieldKind::Unit {
        quote! { 0 }
    } else {
        let calls = fields.fields.iter().map(|Field { name, ty, id, .. }| {
            let id = proc_macro2::Literal::u32_unsuffixed(*id);
            let name = proc_macro2::Ident::new(name, Span::call_site());

            if let Type::Option(ty) = &ty {
                let ty = compile_data_type(opts, ty, quote! { v });
                quote! {
                    ::mabo::buf::size_field_option(#id, #name.as_ref(), |v| { #ty })
                }
            } else {
                let ty = compile_data_type(opts, ty, name.into_token_stream());
                quote! { ::mabo::buf::size_field(#id, || { #ty }) }
            }
        });

        quote! {
           #(#calls +)*
           ::mabo::buf::END_MARKER_SIZE
        }
    }
}

fn compile_generics(types: &[&str]) -> (TokenStream, TokenStream) {
    (!types.is_empty())
        .then(|| {
            let types = types.iter().map(|ty| Ident::new(ty, Span::call_site()));
            let types2 = types.clone();

            (
                quote! { <#(#types,)*> },
                quote! { where #(#types2: ::mabo::buf::Size,)* },
            )
        })
        .unwrap_or_default()
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn compile_data_type(opts: &Opts, ty: &Type<'_>, name: TokenStream) -> TokenStream {
    match &ty {
        Type::Bool => quote! { ::mabo::buf::size_bool(*#name) },
        Type::U8 => quote! { ::mabo::buf::size_u8(*#name) },
        Type::U16 => quote! { ::mabo::buf::size_u16(*#name) },
        Type::U32 => quote! { ::mabo::buf::size_u32(*#name) },
        Type::U64 => quote! { ::mabo::buf::size_u64(*#name) },
        Type::U128 => quote! { ::mabo::buf::size_u128(*#name) },
        Type::I8 => quote! { ::mabo::buf::size_i8(*#name) },
        Type::I16 => quote! { ::mabo::buf::size_i16(*#name) },
        Type::I32 => quote! { ::mabo::buf::size_i32(*#name) },
        Type::I64 => quote! { ::mabo::buf::size_i64(*#name) },
        Type::I128 => quote! { ::mabo::buf::size_i128(*#name) },
        Type::F32 => quote! { ::mabo::buf::size_f32(*#name) },
        Type::F64 => quote! { ::mabo::buf::size_f64(*#name) },
        Type::String | Type::StringRef | Type::BoxString => {
            quote! { ::mabo::buf::size_string(#name) }
        }
        Type::Bytes | Type::BytesRef | Type::BoxBytes => match opts.bytes_type {
            BytesType::VecU8 => quote! { ::mabo::buf::size_bytes_std(#name) },
            BytesType::Bytes => quote! { ::mabo::buf::size_bytes_bytes(#name) },
        },
        Type::Vec(ty) => {
            let ty = compile_data_type(opts, ty, quote! { v });
            quote! { ::mabo::buf::size_vec(#name, |v| { #ty }) }
        }
        Type::HashMap(kv) => {
            let ty_k = compile_data_type(opts, &kv.0, quote! { k });
            let ty_v = compile_data_type(opts, &kv.1, quote! { v });
            quote! { ::mabo::buf::size_hash_map(#name, |k| { #ty_k }, |v| { #ty_v }) }
        }
        Type::HashSet(ty) => {
            let ty = compile_data_type(opts, ty, quote! { v });
            quote! { ::mabo::buf::size_hash_set(#name, |v| { #ty }) }
        }
        Type::Option(ty) => {
            let ty = compile_data_type(opts, ty, quote! { v });
            quote! { ::mabo::buf::size_option(#name.as_ref(), |v| { #ty }) }
        }
        Type::NonZero(ty) => match &**ty {
            Type::U8 => quote! { ::mabo::buf::size_u8(#name.get()) },
            Type::U16 => quote! { ::mabo::buf::size_u16(#name.get()) },
            Type::U32 => quote! { ::mabo::buf::size_u32(#name.get()) },
            Type::U64 => quote! { ::mabo::buf::size_u64(#name.get()) },
            Type::U128 => quote! { ::mabo::buf::size_u128(#name.get()) },
            Type::I8 => quote! { ::mabo::buf::size_i8(#name.get()) },
            Type::I16 => quote! { ::mabo::buf::size_i16(#name.get()) },
            Type::I32 => quote! { ::mabo::buf::size_i32(#name.get()) },
            Type::I64 => quote! { ::mabo::buf::size_i64(#name.get()) },
            Type::I128 => quote! { ::mabo::buf::size_i128(#name.get()) },
            Type::String
            | Type::StringRef
            | Type::Bytes
            | Type::BytesRef
            | Type::Vec(_)
            | Type::HashMap(_)
            | Type::HashSet(_) => compile_data_type(opts, ty, quote! { #name.get() }),
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        Type::Tuple(types) => match types.len() {
            2..=12 => {
                let types = types.iter().enumerate().map(|(idx, ty)| {
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    compile_data_type(opts, ty, quote! { &#name.#idx })
                });
                quote! { #(#types)+* }
            }
            n => todo!("compiler should catch invalid tuple with {n} elements"),
        },
        Type::Array(ty, _size) => {
            let ty = compile_data_type(opts, ty, quote! { v });
            quote! { ::mabo::buf::size_array(#name, |v| { #ty }) }
        }
        Type::External(_) => {
            quote! { #name.size() }
        }
    }
}
