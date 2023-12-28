use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use stef_compiler::simplify::{Enum, Field, FieldKind, Fields, Struct, Type, Variant};

use crate::{BytesType, Opts};

pub(super) fn compile_struct(
    opts: &Opts,
    Struct {
        comment: _,
        name,
        generics,
        fields,
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
    let fields = compile_fields(opts, fields);

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Encode for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::deref_addrof,
                clippy::explicit_auto_deref,
                clippy::needless_borrow,
                clippy::too_many_lines,
            )]
            fn encode(&self, w: &mut impl ::stef::BufMut) {
                let Self #names = self;
                #fields
            }
        }
    }
}

pub(super) fn compile_enum(
    opts: &Opts,
    Enum {
        comment: _,
        name,
        generics,
        variants,
    }: &Enum<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let variants = variants.iter().map(|v| compile_variant(opts, v));

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Encode for #name #generics #generics_where {
            #[allow(
                clippy::borrow_deref_ref,
                clippy::deref_addrof,
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
    let id = quote! { ::stef::VariantId::new(#id) };
    let name = Ident::new(name, Span::call_site());
    let fields_body = compile_fields(opts, fields);
    let field_names = fields
        .fields
        .iter()
        .map(|field| Ident::new(&field.name, Span::call_site()));

    match fields.kind {
        FieldKind::Named => quote! {
            Self::#name{ #(#field_names,)* } => {
                ::stef::buf::encode_variant_id(w, #id);
                #fields_body
            }
        },
        FieldKind::Unnamed => quote! {
            Self::#name(#(#field_names,)*) => {
                ::stef::buf::encode_variant_id(w, #id);
                #fields_body
            }
        },
        FieldKind::Unit => quote! {
            Self::#name => {
                ::stef::buf::encode_variant_id(w, #id);
            }
        },
    }
}

fn compile_fields(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
    if fields.kind == FieldKind::Unit {
        quote! {}
    } else {
        let calls = fields.fields.iter().map(|Field { name, ty, id, .. }| {
            let id = proc_macro2::Literal::u32_unsuffixed(*id);
            let name = proc_macro2::Ident::new(name, Span::call_site());

            if let Type::Option(ty) = &ty {
                let (enc, ty) = compile_data_type(opts, ty, quote! { v }, true);
                let id = quote! { ::stef::FieldId::new(#id, #enc) };
                quote! { ::stef::buf::encode_field_option(w, #id, #name, |w, v| { #ty; }); }
            } else {
                let (enc, ty) = compile_data_type(opts, ty, name.into_token_stream(), true);
                let id = quote! { ::stef::FieldId::new(#id, #enc) };
                quote! { ::stef::buf::encode_field(w, #id, |w| { #ty; }); }
            }
        });

        quote! {
           #(#calls)*
           ::stef::buf::encode_u32(w, ::stef::buf::END_MARKER);
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
                quote! { where #(#types2: ::stef::buf::Encode + ::stef::buf::Size,)* },
            )
        })
        .unwrap_or_default()
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn compile_data_type(
    opts: &Opts,
    ty: &Type<'_>,
    name: TokenStream,
    root: bool,
) -> (TokenStream, TokenStream) {
    match &ty {
        Type::Bool => (
            quote! { ::stef::FieldEncoding::Fixed1 },
            quote! { ::stef::buf::encode_bool(w, *#name) },
        ),
        Type::U8 => (
            quote! { ::stef::FieldEncoding::Fixed1 },
            quote! { ::stef::buf::encode_u8(w, *#name) },
        ),
        Type::U16 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_u16(w, *#name) },
        ),
        Type::U32 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_u32(w, *#name) },
        ),
        Type::U64 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_u64(w, *#name) },
        ),
        Type::U128 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_u128(w, *#name) },
        ),
        Type::I8 => (
            quote! { ::stef::FieldEncoding::Fixed1 },
            quote! { ::stef::buf::encode_i8(w, *#name) },
        ),
        Type::I16 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_i16(w, *#name) },
        ),
        Type::I32 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_i32(w, *#name) },
        ),
        Type::I64 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_i64(w, *#name) },
        ),
        Type::I128 => (
            quote! { ::stef::FieldEncoding::Varint },
            quote! { ::stef::buf::encode_i128(w, *#name) },
        ),
        Type::F32 => (
            quote! { ::stef::FieldEncoding::Fixed4 },
            quote! { ::stef::buf::encode_f32(w, *#name) },
        ),
        Type::F64 => (
            quote! { ::stef::FieldEncoding::Fixed8 },
            quote! { ::stef::buf::encode_f64(w, *#name) },
        ),
        Type::String | Type::StringRef | Type::BoxString => (
            quote! { ::stef::FieldEncoding::LengthPrefixed },
            quote! { ::stef::buf::encode_string(w, #name) },
        ),
        Type::Bytes | Type::BytesRef | Type::BoxBytes => match opts.bytes_type {
            BytesType::VecU8 => (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! { ::stef::buf::encode_bytes_std(w, #name) },
            ),
            BytesType::Bytes => (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! { ::stef::buf::encode_bytes_bytes(w, #name) },
            ),
        },
        Type::Vec(ty) => {
            let size = super::size::compile_data_type(opts, ty, quote! { v });
            let (_, encode) = compile_data_type(opts, ty, quote! { v }, false);
            (quote! { ::stef::FieldEncoding::LengthPrefixed }, {
                quote! { ::stef::buf::encode_vec(w, #name, |v| { #size }, |w, v| { #encode; }) }
            })
        }
        Type::HashMap(kv) => {
            let size_k = super::size::compile_data_type(opts, &kv.0, quote! { k });
            let size_v = super::size::compile_data_type(opts, &kv.1, quote! { v });
            let (_, encode_k) = compile_data_type(opts, &kv.0, quote! { k }, false);
            let (_, encode_v) = compile_data_type(opts, &kv.1, quote! { v }, false);
            (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! {
                    ::stef::buf::encode_hash_map(
                        w,
                        #name,
                        |k| { #size_k },
                        |v| { #size_v },
                        |w, k| { #encode_k; },
                        |w, v| { #encode_v; },
                    )
                },
            )
        }
        Type::HashSet(ty) => {
            let size = super::size::compile_data_type(opts, ty, quote! { v });
            let (_, encode) = compile_data_type(opts, ty, quote! { v }, false);
            (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! { ::stef::buf::encode_hash_set(w, #name, |v| { #size }, |w, v| { #encode; }) },
            )
        }
        Type::Option(ty) => {
            let (_, encode) = compile_data_type(opts, ty, quote! { v }, false);
            (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! { ::stef::buf::encode_option(w, #name, |w, v| { #encode; }) },
            )
        }
        Type::NonZero(ty) => match &**ty {
            Type::U8 => (
                quote! { ::stef::FieldEncoding::Fixed1 },
                quote! { ::stef::buf::encode_u8(w, #name.get()) },
            ),
            Type::U16 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_u16(w, #name.get()) },
            ),
            Type::U32 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_u32(w, #name.get()) },
            ),
            Type::U64 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_u64(w, #name.get()) },
            ),
            Type::U128 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_u128(w, #name.get()) },
            ),
            Type::I8 => (
                quote! { ::stef::FieldEncoding::Fixed1 },
                quote! { ::stef::buf::encode_i8(w, #name.get()) },
            ),
            Type::I16 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_i16(w, #name.get()) },
            ),
            Type::I32 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_i32(w, #name.get()) },
            ),
            Type::I64 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_i64(w, #name.get()) },
            ),
            Type::I128 => (
                quote! { ::stef::FieldEncoding::Varint },
                quote! { ::stef::buf::encode_i128(w, #name.get()) },
            ),
            Type::String
            | Type::StringRef
            | Type::Bytes
            | Type::BytesRef
            | Type::Vec(_)
            | Type::HashMap(_)
            | Type::HashSet(_) => compile_data_type(opts, ty, quote! { #name.get() }, false),
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        Type::Tuple(types) => match types.len() {
            2..=12 => {
                let encode = types.iter().enumerate().map(|(idx, ty)| {
                    let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                    compile_data_type(opts, ty, quote! { &#name.#idx }, false).1
                });
                (
                    quote! { ::stef::FieldEncoding::LengthPrefixed },
                    if root {
                        let size = types.iter().enumerate().map(|(idx, ty)| {
                            let idx = proc_macro2::Literal::usize_unsuffixed(idx);
                            super::size::compile_data_type(opts, ty, quote! { &#name.#idx })
                        });

                        quote! { ::stef::buf::encode_tuple(w, || { #(#size)+* }, |w| { #(#encode;)* }) }
                    } else {
                        quote! { #(#encode;)* }
                    },
                )
            }
            n => todo!("compiler should catch invalid tuple with {n} elements"),
        },
        Type::Array(ty, _size) => {
            let size = super::size::compile_data_type(opts, ty, quote! { v });
            let (_, encode) = compile_data_type(opts, ty, quote! { v }, false);
            (
                quote! { ::stef::FieldEncoding::LengthPrefixed },
                quote! { ::stef::buf::encode_array(w, #name, |v| { #size }, |w, v| { #encode; }) },
            )
        }
        Type::External(_) => (
            quote! { ::stef::FieldEncoding::LengthPrefixed },
            quote! { #name.encode(w) },
        ),
    }
}
