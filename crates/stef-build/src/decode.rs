use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use stef_compiler::simplify::{
    Enum, ExternalType, Field, FieldKind, Fields, Struct, Type, Variant,
};

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
    let name = Ident::new(name, Span::call_site());
    let (generics, generics_where) = compile_generics(generics);
    let field_vars = compile_field_vars(opts, &fields.fields);
    let field_matches = compile_field_matches(opts, fields);
    let field_assigns = compile_field_assigns(fields);

    let body = if fields.kind == FieldKind::Unit {
        quote! { Ok(Self) }
    } else {
        quote! {
            #field_vars

            loop {
                let id = ::stef::buf::decode_id(r)?;
                match id.value {
                    ::stef::buf::END_MARKER => break,
                    #field_matches
                    _ => ::stef::buf::decode_skip(r, id.encoding)?,
                }
            }

            Ok(Self #field_assigns)
        }
    };

    quote! {
        #[automatically_derived]
        impl #generics ::stef::Decode for #name #generics #generics_where {
            #[allow(clippy::type_complexity, clippy::too_many_lines)]
            fn decode(r: &mut impl ::stef::Buf) -> ::stef::buf::Result<Self> {
                #body
            }
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
        impl #generics ::stef::Decode for #name #generics #generics_where {
            #[allow(clippy::too_many_lines)]
            fn decode(r: &mut impl ::stef::Buf) -> ::stef::buf::Result<Self> {
                match ::stef::buf::decode_variant_id(r)?.value {
                    #(#variants,)*
                    id => Err(::stef::buf::Error::UnknownVariant(id)),
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
    let field_vars = compile_field_vars(opts, &fields.fields);
    let field_matches = compile_field_matches(opts, fields);
    let field_assigns = compile_field_assigns(fields);

    if fields.kind == FieldKind::Unit {
        quote! { #id => Ok(Self::#name) }
    } else {
        quote! {
            #id => {
                #field_vars

                loop {
                    let id = ::stef::buf::decode_id(r)?;
                    match id.value {
                        ::stef::buf::END_MARKER => break,
                        #field_matches
                        _ => ::stef::buf::decode_skip(r, id.encoding)?,
                    }
                }

                Ok(Self::#name #field_assigns)
            }
        }
    }
}

fn compile_field_vars(opts: &Opts, fields: &[Field<'_>]) -> TokenStream {
    let vars = fields.iter().map(|field| {
        let name = Ident::new(&field.name, Span::call_site());
        (name, &field.ty)
    });

    let vars = vars.map(|(name, ty)| {
        let ty_ident = super::definition::compile_data_type(opts, ty);

        if matches!(ty, Type::Option(_)) {
            quote! { let mut #name: #ty_ident = None; }
        } else {
            quote! { let mut #name: Option<#ty_ident> = None; }
        }
    });

    quote! { #(#vars)* }
}

fn compile_field_matches(opts: &Opts, fields: &Fields<'_>) -> TokenStream {
    let calls = fields.fields.iter().map(|Field { name, ty, id, .. }| {
        let id = proc_macro2::Literal::u32_unsuffixed(*id);
        let name = proc_macro2::Ident::new(name, Span::call_site());
        let ty = compile_data_type(opts, if let Type::Option(ty) = &ty { ty } else { ty }, true);

        quote! { #id => #name = Some(#ty?) }
    });

    quote! { #(#calls,)* }
}

fn compile_field_assigns(fields: &Fields<'_>) -> TokenStream {
    let assigns = fields.fields.iter().map(|Field { name, ty, id, .. }| {
        let name_lit = if fields.kind == FieldKind::Named {
            let lit = proc_macro2::Literal::string(name);
            quote! { Some(#lit)}
        } else {
            quote! { None }
        };
        let name = Ident::new(name, Span::call_site());
        let id = proc_macro2::Literal::u32_unsuffixed(*id);

        if matches!(ty, Type::Option(_)) {
            quote! { #name }
        } else if fields.kind == FieldKind::Named {
            quote! {
                #name: #name.ok_or(::stef::buf::Error::MissingField {
                    id: #id,
                    name: #name_lit,
                })?
            }
        } else {
            quote! {
                #name.ok_or(::stef::buf::Error::MissingField {
                   id: #id,
                   name: #name_lit,
               })?
            }
        }
    });

    if fields.kind == FieldKind::Named {
        quote! { { #(#assigns,)* } }
    } else {
        quote! { (#(#assigns,)*) }
    }
}

fn compile_generics(types: &[&str]) -> (TokenStream, TokenStream) {
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

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn compile_data_type(opts: &Opts, ty: &Type<'_>, root: bool) -> TokenStream {
    match ty {
        Type::Bool => quote! { ::stef::buf::decode_bool(r) },
        Type::U8 => quote! { ::stef::buf::decode_u8(r) },
        Type::U16 => quote! { ::stef::buf::decode_u16(r) },
        Type::U32 => quote! { ::stef::buf::decode_u32(r) },
        Type::U64 => quote! { ::stef::buf::decode_u64(r) },
        Type::U128 => quote! { ::stef::buf::decode_u128(r) },
        Type::I8 => quote! { ::stef::buf::decode_i8(r) },
        Type::I16 => quote! { ::stef::buf::decode_i16(r) },
        Type::I32 => quote! { ::stef::buf::decode_i32(r) },
        Type::I64 => quote! { ::stef::buf::decode_i64(r) },
        Type::I128 => quote! { ::stef::buf::decode_i128(r) },
        Type::F32 => quote! { ::stef::buf::decode_f32(r) },
        Type::F64 => quote! { ::stef::buf::decode_f64(r) },
        Type::String | Type::StringRef => quote! { ::stef::buf::decode_string(r) },
        Type::Bytes | Type::BytesRef => match opts.bytes_type {
            BytesType::VecU8 => quote! { ::stef::buf::decode_bytes_std(r) },
            BytesType::Bytes => quote! { ::stef::buf::decode_bytes_bytes(r) },
        },
        Type::Vec(ty) => {
            let ty = compile_data_type(opts, ty, false);
            quote! { ::stef::buf::decode_vec(r, |r| { #ty }) }
        }
        Type::HashMap(kv) => {
            let ty_k = compile_data_type(opts, &kv.0, false);
            let ty_v = compile_data_type(opts, &kv.1, false);
            quote! { ::stef::buf::decode_hash_map(r, |r| { #ty_k }, |r| { #ty_v }) }
        }
        Type::HashSet(ty) => {
            let ty = compile_data_type(opts, ty, false);
            quote! { ::stef::buf::decode_hash_set(r, |r| { #ty }) }
        }
        Type::Option(ty) => {
            let ty = compile_data_type(opts, ty, false);
            quote! { ::stef::buf::decode_option(r, |r| { #ty }) }
        }
        Type::NonZero(ty) => match &**ty {
            Type::U8 => quote! { ::stef::buf::decode_non_zero_u8(r) },
            Type::U16 => quote! { ::stef::buf::decode_non_zero_u16(r) },
            Type::U32 => quote! { ::stef::buf::decode_non_zero_u32(r) },
            Type::U64 => quote! { ::stef::buf::decode_non_zero_u64(r) },
            Type::U128 => quote! { ::stef::buf::decode_non_zero_u128(r) },
            Type::I8 => quote! { ::stef::buf::decode_non_zero_i8(r) },
            Type::I16 => quote! { ::stef::buf::decode_non_zero_i16(r) },
            Type::I32 => quote! { ::stef::buf::decode_non_zero_i32(r) },
            Type::I64 => quote! { ::stef::buf::decode_non_zero_i64(r) },
            Type::I128 => quote! { ::stef::buf::decode_non_zero_i128(r) },
            Type::String | Type::StringRef => {
                quote! { ::stef::buf::decode_non_zero_string(r) }
            }
            Type::Bytes | Type::BytesRef => match opts.bytes_type {
                BytesType::VecU8 => {
                    quote! { ::stef::buf::decode_non_zero_bytes_std(r) }
                }
                BytesType::Bytes => {
                    quote! { ::stef::buf::decode_non_zero_bytes_bytes(r) }
                }
            },
            Type::Vec(ty) => {
                let ty = compile_data_type(opts, ty, false);
                quote! { ::stef::buf::decode_non_zero_vec(r, |r| { #ty }) }
            }
            Type::HashMap(kv) => {
                let ty_k = compile_data_type(opts, &kv.0, false);
                let ty_v = compile_data_type(opts, &kv.1, false);
                quote! { ::stef::buf::decode_non_zero_hash_map(r, |r| { #ty_k }, |r| { #ty_v }) }
            }
            Type::HashSet(ty) => {
                let ty = compile_data_type(opts, ty, false);
                quote! { ::stef::buf::decode_non_zero_hash_set(r, |r| { #ty }) }
            }
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        Type::BoxString => quote! { Box::<str>::decode(r) },
        Type::BoxBytes => quote! { Box::<[u8]>::decode(r) },
        Type::Tuple(types) => match types.len() {
            2..=12 => {
                let types = types.iter().map(|ty| compile_data_type(opts, ty, false));
                let length = root.then_some(quote! { ::stef::buf::decode_u64(r)?; });
                quote! { {
                    #length
                    Ok::<_, ::stef::buf::Error>((#(#types?,)*))
                } }
            }
            n => todo!("compiler should catch invalid tuple with {n} elements"),
        },
        Type::Array(ty, _size) => {
            let ty = compile_data_type(opts, ty, false);
            quote! { ::stef::buf::decode_array(r, |r| { #ty }) }
        }
        Type::External(ExternalType {
            path,
            name,
            generics,
        }) => {
            let path = path.iter().map(|part| Ident::new(part, Span::call_site()));
            let ty = Ident::new(name, Span::call_site());
            let generics = (!generics.is_empty()).then(|| {
                let types = generics
                    .iter()
                    .map(|ty| super::definition::compile_data_type(opts, ty));
                quote! { ::<#(#types,)*> }
            });
            quote! { #(#path::)* #ty #generics::decode(r) }
        }
    }
}
