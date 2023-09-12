use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use stef_parser::{
    Comment, Const, DataType, Definition, Enum, ExternalType, Fields, Generics, Import, Literal,
    Module, NamedField, Schema, Struct, TypeAlias, UnnamedField, Variant,
};

use super::encode;

pub(crate) fn compile_schema(Schema { definitions }: &Schema<'_>) -> TokenStream {
    let definitions = definitions.iter().map(compile_definition);

    quote! { #(#definitions)* }
}

fn compile_definition(definition: &Definition<'_>) -> TokenStream {
    let definition = match definition {
        Definition::Module(m) => compile_module(m),
        Definition::Struct(s) => {
            let def = compile_struct(s);
            let encode = encode::compile_struct(s);

            quote! {
                #def
                #encode
            }
        }
        Definition::Enum(e) => {
            let def = compile_enum(e);
            let encode = encode::compile_enum(e);

            quote! {
                #def
                #encode
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
    let name = Ident::new(name, Span::call_site());
    let definitions = definitions.iter().map(compile_definition);

    quote! {
        #comment
        pub mod #name {
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
    let name = Ident::new(name, Span::call_site());
    let generics = compile_generics(generics);
    let fields = compile_fields(fields, true);

    quote! {
        #comment
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
    let name = Ident::new(name, Span::call_site());
    let generics = compile_generics(generics);
    let variants = variants.iter().map(compile_variant);

    quote! {
        #comment
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
    }: &Variant<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let name = Ident::new(name, Span::call_site());
    let fields = compile_fields(fields, false);

    quote! {
        #comment
        #name #fields
    }
}

fn compile_alias(
    TypeAlias {
        comment,
        alias,
        target,
    }: &TypeAlias<'_>,
) -> TokenStream {
    let comment = compile_comment(comment);
    let alias = compile_data_type(alias);
    let target = compile_data_type(target);

    quote! {
        #comment
        pub type #alias = #target;
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
    let name = Ident::new(name, Span::call_site());
    let ty = compile_const_data_type(ty);
    let value = compile_literal(value);

    quote! {
        #comment
        const #name: #ty = #value;
    }
}

fn compile_import(Import { segments, element }: &Import<'_>) -> TokenStream {
    let segments = segments.iter().enumerate().map(|(i, segment)| {
        let segment = Ident::new(segment, Span::call_site());
        if i > 0 {
            quote! {::#segment}
        } else {
            quote! {#segment}
        }
    });
    let element = element.map(|element| {
        let element = Ident::new(element, Span::call_site());
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
    (!types.is_empty()).then(|| quote! { <#(#types,)*> })
}

fn compile_fields(fields: &Fields<'_>, public: bool) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let fields = named.iter().map(
                |NamedField {
                     comment,
                     name,
                     ty,
                     id: _,
                 }| {
                    let comment = compile_comment(comment);
                    let public = public.then(|| quote! { pub });
                    let name = Ident::new(name, Span::call_site());
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
            let fields = unnamed.iter().map(|UnnamedField { ty, id: _ }| {
                let ty = compile_data_type(ty);
                quote! { #ty }
            });

            quote! { (#(#fields,)*) }
        }
        Fields::Unit => quote! {},
    }
}

fn compile_data_type(ty: &DataType<'_>) -> TokenStream {
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
            quote! { HashMap<#k, #v> }
        }
        DataType::HashSet(ty) => {
            let ty = compile_data_type(ty);
            quote! { HashSet<#ty> }
        }
        DataType::Option(ty) => {
            let ty = compile_data_type(ty);
            quote! { Option<#ty> }
        }
        DataType::NonZero(ty) => match **ty {
            DataType::U8 => quote! { NonZeroU8 },
            DataType::U16 => quote! { NonZeroU16 },
            DataType::U32 => quote! { NonZeroU32 },
            DataType::U64 => quote! { NonZeroU64 },
            DataType::U128 => quote! { NonZeroU128 },
            DataType::I8 => quote! { NonZeroI8 },
            DataType::I16 => quote! { NonZeroI16 },
            DataType::I32 => quote! { NonZeroI32 },
            DataType::I64 => quote! { NonZeroI64 },
            DataType::I128 => quote! { NonZeroI128 },
            _ => compile_data_type(ty),
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
            let name = Ident::new(name, Span::call_site());
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    fn parse(input: &str, expect: &str) {
        let parsed = Schema::parse(input).unwrap();
        println!("==========\n{parsed}");

        let compiled = compile_schema(&parsed);
        println!("----------\n{compiled}");

        let pretty = prettyplease::unparse(&syn::parse2(compiled.clone()).unwrap());
        println!("----------\n{pretty}==========");

        assert_eq!(expect, pretty);
    }

    #[test]
    fn basic_module() {
        let input = indoc! {r#"
            /// Hello world!
            mod sample {}
        "#};
        let expect = indoc! {r#"
            /// Hello world!
            pub mod sample {}
        "#};

        parse(input, expect);
    }

    #[test]
    fn basic_struct() {
        let input = indoc! {r#"
            /// Hello world!
            struct Sample {
                field1: u32 @1,
                field2: bytes @2,
                field3: (bool, [i16; 4]) @3,
            }
        "#};
        let expect = indoc! {r#"
            /// Hello world!
            pub struct Sample {
                pub field1: u32,
                pub field2: Vec<u8>,
                pub field3: (bool, [i16; 4]),
            }
            impl ::stef::Encode for Sample {
                fn encode(&self, w: &mut impl ::stef::BufMut) {
                    ::stef::write_field(w, 1, |w| { ::stef::encode_u32(w, self.field1) });
                    ::stef::write_field(w, 2, |w| { ::stef::encode_bytes(w, &self.field2) });
                    ::stef::write_field(w, 3, |w| { ::stef::write_tuple2(w, &self.field3) });
                }
            }
        "#};

        parse(input, expect);
    }

    #[test]
    fn basic_enum() {
        let input = indoc! {r#"
            /// Hello world!
            enum Sample {
                Variant1 @1,
                Variant2(u32 @1, u8 @2) @2,
                Variant3 {
                    field1: string @1,
                    field2: vec<bool> @2,
                } @3,
            }
        "#};
        let expect = indoc! {r#"
            /// Hello world!
            pub enum Sample {
                Variant1,
                Variant2(u32, u8),
                Variant3 { field1: String, field2: Vec<bool> },
            }
            impl ::stef::Encode for Sample {
                fn encode(&self, w: &mut impl ::stef::BufMut) {
                    match self {
                        Self::Variant1 => {
                            ::stef::write_id(w, 1);
                        }
                        Self::Variant2(n0, n1) => {
                            ::stef::write_id(w, 2);
                            ::stef::write_field(w, 1, |w| { ::stef::encode_u32(w, n0) });
                            ::stef::write_field(w, 2, |w| { ::stef::encode_u8(w, n1) });
                        }
                        Self::Variant3 { field1, field2 } => {
                            ::stef::write_id(w, 3);
                            ::stef::write_field(w, 1, |w| { ::stef::encode_string(w, &field1) });
                            ::stef::write_field(w, 2, |w| { ::stef::encode_vec(w, &field2) });
                        }
                    }
                }
            }
        "#};

        parse(input, expect);
    }

    #[test]
    fn basic_alias() {
        let input = indoc! {r#"
            /// Hello world!
            type Sample = String;
        "#};
        let expect = indoc! {r#"
            /// Hello world!
            pub type Sample = String;
        "#};

        parse(input, expect);
    }

    #[test]
    fn basic_const() {
        let input = indoc! {r#"
            /// A bool.
            const BOOL: bool = true;
            /// An integer.
            const INT: u32 = 100;
            /// A float.
            const FLOAT: f64 = 5.0;
            /// A string.
            const STRING: string = "hello";
            /// Some bytes.
            const BYTES: bytes = [1, 2, 3];
        "#};
        let expect = indoc! {r#"
            /// A bool.
            const BOOL: bool = true;
            /// An integer.
            const INT: u32 = 100;
            /// A float.
            const FLOAT: f64 = 5.0;
            /// A string.
            const STRING: &str = "hello";
            /// Some bytes.
            const BYTES: &[u8] = b"\x01\x02\x03";
        "#};

        parse(input, expect);
    }

    #[test]
    fn basic_import() {
        let input = indoc! {r#"
            use other::module;
            use other::module::Type;
        "#};
        let expect = indoc! {r#"
            use other::module;
            use other::module::Type;
        "#};

        parse(input, expect);
    }
}
