use mabo_compiler::simplify::{
    Const, Definition, Enum, ExternalType, Field, FieldKind, Fields, Import, Literal, Module,
    Schema, Struct, Type, TypeAlias, Variant,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};

use super::{decode, encode, size};
use crate::{BytesType, Opts};

/// Take a single schema and convert it into Rust source code.
#[must_use]
pub fn compile_schema(opts: &Opts, Schema { definitions, .. }: &Schema<'_>) -> TokenStream {
    let definitions = definitions.iter().map(|def| compile_definition(opts, def));

    quote! {
        #[allow(unused_imports)]
        use ::mabo::buf::{Decode, Encode, Size};

        #(#definitions)*
    }
}

fn compile_definition(opts: &Opts, definition: &Definition<'_>) -> TokenStream {
    match definition {
        Definition::Module(m) => compile_module(opts, m),
        Definition::Struct(s) => {
            let def = compile_struct(opts, s);
            let encode = encode::compile_struct(opts, s);
            let decode = decode::compile_struct(opts, s);
            let size = size::compile_struct(opts, s);

            quote! {
                #def
                #encode
                #decode
                #size
            }
        }
        Definition::Enum(e) => {
            let def = compile_enum(opts, e);
            let encode = encode::compile_enum(opts, e);
            let decode = decode::compile_enum(opts, e);
            let size = size::compile_enum(opts, e);

            quote! {
                #def
                #encode
                #decode
                #size
            }
        }
        Definition::TypeAlias(a) => compile_alias(opts, a),
        Definition::Const(c) => compile_const(c),
        Definition::Import(i) => compile_import(i),
    }
}

fn compile_module(
    opts: &Opts,
    Module {
        comment,
        name,
        definitions,
        ..
    }: &Module<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let definitions = definitions.iter().map(|def| compile_definition(opts, def));

    quote! {
        #comment
        pub mod #name {
            #[allow(unused_imports)]
            use ::mabo::buf::{Decode, Encode, Size};

            #(#definitions)*
        }
    }
}

fn compile_struct(
    opts: &Opts,
    Struct {
        comment,
        name,
        generics,
        fields,
        ..
    }: &Struct<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let generics = compile_generics(generics);
    let semicolon = (fields.kind != FieldKind::Named).then_some(quote! {;});
    let fields = compile_fields(opts, fields, true);

    quote! {
        #comment
        #[derive(Clone, Debug, PartialEq)]
        #[allow(clippy::module_name_repetitions, clippy::option_option)]
        pub struct #name #generics #fields #semicolon
    }
}

fn compile_enum(
    opts: &Opts,
    Enum {
        comment,
        name,
        generics,
        variants,
        ..
    }: &Enum<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let generics = compile_generics(generics);
    let variants = variants.iter().map(|v| compile_variant(opts, v));

    quote! {
        #comment
        #[derive(Clone, Debug, PartialEq)]
        #[allow(clippy::module_name_repetitions, clippy::option_option)]
        pub enum #name #generics {
            #(#variants,)*
        }
    }
}

fn compile_variant(
    opts: &Opts,
    Variant {
        comment,
        name,
        fields,
        ..
    }: &Variant<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let fields = compile_fields(opts, fields, false);

    quote! {
        #comment
        #name #fields
    }
}

fn compile_alias(
    opts: &Opts,
    TypeAlias {
        comment,
        name,
        generics,
        target,
        ..
    }: &TypeAlias<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let generics = compile_generics(generics);
    let target = compile_data_type(opts, target);

    quote! {
        #comment
        #[allow(dead_code, clippy::module_name_repetitions, clippy::option_option)]
        pub type #name #generics = #target;
    }
}

fn compile_const(
    Const {
        comment,
        name,
        ty,
        value,
        ..
    }: &Const<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let ty = compile_const_data_type(ty);
    let value = compile_literal(value);

    quote! {
        #comment
        #[allow(dead_code)]
        pub const #name: #ty = #value;
    }
}

fn compile_import(
    Import {
        segments, element, ..
    }: &Import<'_>,
) -> TokenStream {
    let segments = segments.iter().enumerate().map(|(i, segment)| {
        let segment = Ident::new(segment, Span::call_site());
        if i > 0 {
            quote! {::#segment}
        } else {
            quote! {#segment}
        }
    });
    let element = element.as_ref().map(|element| {
        let element = Ident::new(element, Span::call_site());
        quote! { ::#element}
    });

    quote! {
        #[allow(unused_imports)]
        use super::#(#segments)*#element;
    }
}

fn compile_comment(lines: &[&str]) -> TokenStream {
    let lines = lines.iter().map(|line| format!(" {line}"));
    quote! { #(#[doc = #lines])* }
}

fn compile_generics(types: &[&str]) -> Option<TokenStream> {
    (!types.is_empty()).then(|| {
        let types = types.iter().map(|ty| Ident::new(ty, Span::call_site()));
        quote! { <#(#types,)*> }
    })
}

fn compile_fields(opts: &Opts, fields: &Fields<'_>, for_struct: bool) -> TokenStream {
    let values = fields.fields.iter().map(
        |Field {
             comment, name, ty, ..
         }| {
            let public = for_struct.then(|| quote! { pub });
            let ty = compile_data_type(opts, ty);

            if fields.kind == FieldKind::Named {
                let comment = compile_comment(comment);
                let name = Ident::new(name, Span::call_site());

                quote! {
                    #comment
                    #public #name: #ty
                }
            } else {
                quote! { #public #ty }
            }
        },
    );

    match fields.kind {
        FieldKind::Named => quote! { {#(#values,)*} },
        FieldKind::Unnamed => quote! { (#(#values,)*) },
        FieldKind::Unit => quote! {},
    }
}

pub(super) fn compile_data_type(opts: &Opts, ty: &Type<'_>) -> TokenStream {
    match &ty {
        Type::Bool => quote! { bool },
        Type::U8 => quote! { u8 },
        Type::U16 => quote! { u16 },
        Type::U32 => quote! { u32 },
        Type::U64 => quote! { u64 },
        Type::U128 => quote! { u128 },
        Type::I8 => quote! { i8 },
        Type::I16 => quote! { i16 },
        Type::I32 => quote! { i32 },
        Type::I64 => quote! { i64 },
        Type::I128 => quote! { i128 },
        Type::F32 => quote! { f32 },
        Type::F64 => quote! { f64 },
        Type::String | Type::StringRef => quote! { String },
        Type::Bytes | Type::BytesRef => match opts.bytes_type {
            BytesType::VecU8 => quote! { Vec<u8> },
            BytesType::Bytes => quote! { ::mabo::buf::Bytes },
        },
        Type::Vec(ty) => {
            let ty = compile_data_type(opts, ty);
            quote! { Vec<#ty> }
        }
        Type::HashMap(kv) => {
            let k = compile_data_type(opts, &kv.0);
            let v = compile_data_type(opts, &kv.1);
            quote! { ::std::collections::HashMap<#k, #v> }
        }
        Type::HashSet(ty) => {
            let ty = compile_data_type(opts, ty);
            quote! { ::std::collections::HashSet<#ty> }
        }
        Type::Option(ty) => {
            let ty = compile_data_type(opts, ty);
            quote! { Option<#ty> }
        }
        Type::NonZero(ty) => match &**ty {
            Type::U8 => quote! { ::std::num::NonZeroU8 },
            Type::U16 => quote! { ::std::num::NonZeroU16 },
            Type::U32 => quote! { ::std::num::NonZeroU32 },
            Type::U64 => quote! { ::std::num::NonZeroU64 },
            Type::U128 => quote! { ::std::num::NonZeroU128 },
            Type::I8 => quote! { ::std::num::NonZeroI8 },
            Type::I16 => quote! { ::std::num::NonZeroI16 },
            Type::I32 => quote! { ::std::num::NonZeroI32 },
            Type::I64 => quote! { ::std::num::NonZeroI64 },
            Type::I128 => quote! { ::std::num::NonZeroI128 },
            Type::String | Type::StringRef => quote! { ::mabo::NonZeroString },
            Type::Bytes | Type::BytesRef => match opts.bytes_type {
                BytesType::VecU8 => quote! { ::mabo::NonZeroBytes },
                BytesType::Bytes => quote! { ::mabo::NonZero<::mabo::buf::Bytes> },
            },
            Type::Vec(ty) => {
                let ty = compile_data_type(opts, ty);
                quote! { ::mabo::NonZeroVec<#ty> }
            }
            Type::HashMap(kv) => {
                let k = compile_data_type(opts, &kv.0);
                let v = compile_data_type(opts, &kv.1);
                quote! { ::mabo::NonZeroHashMap<#k, #v> }
            }
            Type::HashSet(ty) => {
                let ty = compile_data_type(opts, ty);
                quote! { ::mabo::NonZeroHashSet<#ty> }
            }
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        Type::BoxString => quote! { Box<str> },
        Type::BoxBytes => quote! { Box<[u8]> },
        Type::Tuple(types) => {
            let types = types.iter().map(|ty| compile_data_type(opts, ty));
            quote! { (#(#types,)*) }
        }
        Type::Array(ty, size) => {
            let ty = compile_data_type(opts, ty);
            let size = proc_macro2::Literal::u32_unsuffixed(*size);
            quote! { [#ty; #size] }
        }
        Type::External(ExternalType {
            path,
            name,
            generics,
        }) => {
            let path = path.iter().map(|part| Ident::new(part, Span::call_site()));
            let name = Ident::new(name, Span::call_site());
            let generics = (!generics.is_empty()).then(|| {
                let types = generics.iter().map(|ty| compile_data_type(opts, ty));
                quote! { <#(#types,)*> }
            });

            quote! {
                #(#path::)* #name #generics
            }
        }
    }
}

fn compile_const_data_type(ty: &Type<'_>) -> TokenStream {
    match &ty {
        Type::Bool => quote! { bool },
        Type::U8 => quote! { u8 },
        Type::U16 => quote! { u16 },
        Type::U32 => quote! { u32 },
        Type::U64 => quote! { u64 },
        Type::U128 => quote! { u128 },
        Type::I8 => quote! { i8 },
        Type::I16 => quote! { i16 },
        Type::I32 => quote! { i32 },
        Type::I64 => quote! { i64 },
        Type::I128 => quote! { i128 },
        Type::F32 => quote! { f32 },
        Type::F64 => quote! { f64 },
        Type::String | Type::StringRef => quote! { &str },
        Type::Bytes | Type::BytesRef => quote! { &[u8] },
        _ => panic!("invalid data type for const"),
    }
}

fn compile_literal(literal: &Literal) -> TokenStream {
    match &literal {
        Literal::Bool(b) => quote! { #b },
        Literal::Int(i) => proc_macro2::Literal::i128_unsuffixed(*i).into_token_stream(),
        Literal::Float(f) => proc_macro2::Literal::f64_unsuffixed(*f).into_token_stream(),
        Literal::String(s) => proc_macro2::Literal::string(s).into_token_stream(),
        Literal::Bytes(b) => proc_macro2::Literal::byte_string(b).into_token_stream(),
    }
}
