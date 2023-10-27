use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use stef_parser::{
    Comment, Const, DataType, Definition, Enum, ExternalType, Fields, Generics, Import, Literal,
    Module, NamedField, Schema, Struct, TypeAlias, UnnamedField, Variant, Name,
};

use super::{decode, encode};

pub fn compile_schema(Schema { definitions }: &Schema<'_>) -> TokenStream {
    let definitions = definitions.iter().map(compile_definition);

    quote! {
        #[allow(unused_imports)]
        use ::stef::buf::{Decode, Encode};

        #(#definitions)*
    }
}

fn compile_definition(definition: &Definition<'_>) -> TokenStream {
    let definition = match definition {
        Definition::Module(m) => compile_module(m),
        Definition::Struct(s) => {
            let def = compile_struct(s);
            let encode = encode::compile_struct(s);
            let decode = decode::compile_struct(s);

            quote! {
                #def
                #encode
                #decode
            }
        }
        Definition::Enum(e) => {
            let def = compile_enum(e);
            let encode = encode::compile_enum(e);
            let decode = decode::compile_enum(e);

            quote! {
                #def
                #encode
                #decode
            }
        }
        Definition::TypeAlias(a) => compile_alias(a),
        Definition::Const(c) => compile_const(c),
        Definition::Import(i) => compile_import(i),
    };

    quote! { #definition }
}

fn compile_module(
    Module {
        comment,
        name,
        definitions,
    }: &Module<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let definitions = definitions.iter().map(compile_definition);

    quote! {
        #comment
        pub mod #name {
            #[allow(unused_imports)]
            use ::stef::buf::{Decode, Encode};

            #(#definitions)*
        }
    }
}

fn compile_struct(
    Struct {
        comment,
        attributes: _,
        name,
        generics,
        fields,
    }: &Struct<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let generics = compile_generics(generics);
    let fields = compile_fields(fields, true);

    quote! {
        #comment
        #[derive(Clone, Debug, PartialEq)]
        #[allow(clippy::module_name_repetitions, clippy::option_option)]
        pub struct #name #generics #fields
    }
}

fn compile_enum(
    Enum {
        comment,
        attributes: _,
        name,
        generics,
        variants,
    }: &Enum<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let generics = compile_generics(generics);
    let variants = variants.iter().map(compile_variant);

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
    Variant {
        comment,
        name,
        fields,
        id: _,
        ..
    }: &Variant<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let fields = compile_fields(fields, false);

    quote! {
        #comment
        #name #fields
    }
}

fn compile_alias(
    TypeAlias {
        comment,
        name,
        generics,
        target,
    }: &TypeAlias<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let generics = compile_generics(generics);
    let target = compile_data_type(target);

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
    }: &Const<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name.get(), Span::call_site());
    let ty = compile_const_data_type(ty);
    let value = compile_literal(value);

    quote! {
        #comment
        #[allow(dead_code)]
        pub const #name: #ty = #value;
    }
}

fn compile_import(Import { segments, element }: &Import<'_>) -> TokenStream {
    let segments = segments.iter().enumerate().map(|(i, segment)| {
        let segment = Ident::new(segment.get(), Span::call_site());
        if i > 0 {
            quote! {::#segment}
        } else {
            quote! {#segment}
        }
    });
    let element = element.as_ref().map(|element| {
        let element = Ident::new(element.get(), Span::call_site());
        quote! { ::#element}
    });

    quote! {
        use #(#segments)*#element;
    }
}

fn compile_comment(Comment(lines): &Comment<'_>) -> TokenStream {
    let lines = lines.iter().map(|line| format!(" {line}"));
    quote! { #(#[doc = #lines])* }
}

fn compile_generics(Generics(types): &Generics<'_>) -> Option<TokenStream> {
    (!types.is_empty()).then(|| {
        let types = types
            .iter()
            .map(|ty| Ident::new(ty.get(), Span::call_site()));
        quote! { <#(#types,)*> }
    })
}

fn compile_fields(fields: &Fields<'_>, for_struct: bool) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let fields = named.iter().map(
                |NamedField {
                     comment,
                     name,
                     ty,
                     id: _,
                     ..
                 }| {
                    let comment = compile_comment(comment);
                    let public = for_struct.then(|| quote! { pub });
                    let name = Ident::new(name.get(), Span::call_site());
                    let ty = compile_data_type(ty);
                    quote! {
                        #comment
                        #public #name: #ty
                    }
                },
            );

            quote! { {
                #(#fields,)*
            } }
        }
        Fields::Unnamed(unnamed) => {
            let fields = unnamed.iter().map(|UnnamedField { ty, id: _, .. }| {
                let ty = compile_data_type(ty);
                quote! { #ty }
            });

            if for_struct {
                quote! { (#(#fields,)*); }
            } else {
                quote! { (#(#fields,)*) }
            }
        }
        Fields::Unit => {
            if for_struct {
                quote! { ; }
            } else {
                quote! {}
            }
        }
    }
}

pub(super) fn compile_data_type(ty: &DataType<'_>) -> TokenStream {
    match ty {
        DataType::Bool => quote! { bool },
        DataType::U8 => quote! { u8 },
        DataType::U16 => quote! { u16 },
        DataType::U32 => quote! { u32 },
        DataType::U64 => quote! { u64 },
        DataType::U128 => quote! { u128 },
        DataType::I8 => quote! { i8 },
        DataType::I16 => quote! { i16 },
        DataType::I32 => quote! { i32 },
        DataType::I64 => quote! { i64 },
        DataType::I128 => quote! { i128 },
        DataType::F32 => quote! { f32 },
        DataType::F64 => quote! { f64 },
        DataType::String | DataType::StringRef => quote! { String },
        DataType::Bytes | DataType::BytesRef => quote! { Vec<u8> },
        DataType::Vec(ty) => {
            let ty = compile_data_type(ty);
            quote! { Vec<#ty> }
        }
        DataType::HashMap(kv) => {
            let k = compile_data_type(&kv.0);
            let v = compile_data_type(&kv.1);
            quote! { ::std::collections::HashMap<#k, #v> }
        }
        DataType::HashSet(ty) => {
            let ty = compile_data_type(ty);
            quote! { ::std::collections::HashSet<#ty> }
        }
        DataType::Option(ty) => {
            let ty = compile_data_type(ty);
            quote! { Option<#ty> }
        }
        DataType::NonZero(ty) => match &**ty {
            DataType::U8 => quote! { ::std::num::NonZeroU8 },
            DataType::U16 => quote! { ::std::num::NonZeroU16 },
            DataType::U32 => quote! { ::std::num::NonZeroU32 },
            DataType::U64 => quote! { ::std::num::NonZeroU64 },
            DataType::U128 => quote! { ::std::num::NonZeroU128 },
            DataType::I8 => quote! { ::std::num::NonZeroI8 },
            DataType::I16 => quote! { ::std::num::NonZeroI16 },
            DataType::I32 => quote! { ::std::num::NonZeroI32 },
            DataType::I64 => quote! { ::std::num::NonZeroI64 },
            DataType::I128 => quote! { ::std::num::NonZeroI128 },
            DataType::String | DataType::StringRef => quote! { ::stef::NonZeroString },
            DataType::Bytes | DataType::BytesRef => quote! { ::stef::NonZeroBytes },
            DataType::Vec(ty) => {
                let ty = compile_data_type(ty);
                quote! { ::stef::NonZeroVec<#ty> }
            }
            DataType::HashMap(kv) => {
                let k = compile_data_type(&kv.0);
                let v = compile_data_type(&kv.1);
                quote! { ::stef::NonZeroHashMap<#k, #v> }
            }
            DataType::HashSet(ty) => {
                let ty = compile_data_type(ty);
                quote! { ::stef::NonZeroHashSet<#ty> }
            }
            ty => todo!("compiler should catch invalid {ty:?} type"),
        },
        DataType::BoxString => quote! { Box<str> },
        DataType::BoxBytes => quote! { Box<[u8]> },
        DataType::Tuple(types) => {
            let types = types.iter().map(compile_data_type);
            quote! { (#(#types,)*) }
        }
        DataType::Array(ty, size) => {
            let ty = compile_data_type(ty);
            let size = proc_macro2::Literal::u32_unsuffixed(*size);
            quote! { [#ty; #size] }
        }
        DataType::External(ExternalType {
            path,
            name,
            generics,
        }) => {
            let path = path.iter().map(Name::get);
            let name = Ident::new(name.get(), Span::call_site());
            let generics = (!generics.is_empty()).then(|| {
                let types = generics.iter().map(compile_data_type);
                quote! { <#(#types,)*> }
            });

            quote! {
                #(#path::)* #name #generics
            }
        }
    }
}

fn compile_const_data_type(ty: &DataType<'_>) -> TokenStream {
    match ty {
        DataType::Bool => quote! { bool },
        DataType::U8 => quote! { u8 },
        DataType::U16 => quote! { u16 },
        DataType::U32 => quote! { u32 },
        DataType::U64 => quote! { u64 },
        DataType::U128 => quote! { u128 },
        DataType::I8 => quote! { i8 },
        DataType::I16 => quote! { i16 },
        DataType::I32 => quote! { i32 },
        DataType::I64 => quote! { i64 },
        DataType::I128 => quote! { i128 },
        DataType::F32 => quote! { f32 },
        DataType::F64 => quote! { f64 },
        DataType::String | DataType::StringRef => quote! { &str },
        DataType::Bytes | DataType::BytesRef => quote! { &[u8] },
        _ => panic!("invalid data type for const"),
    }
}

fn compile_literal(literal: &Literal) -> TokenStream {
    match literal {
        Literal::Bool(b) => quote! { #b },
        Literal::Int(i) => proc_macro2::Literal::i128_unsuffixed(*i).into_token_stream(),
        Literal::Float(f) => proc_macro2::Literal::f64_unsuffixed(*f).into_token_stream(),
        Literal::String(s) => proc_macro2::Literal::string(s).into_token_stream(),
        Literal::Bytes(b) => proc_macro2::Literal::byte_string(b).into_token_stream(),
    }
}
