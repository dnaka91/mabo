use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use stef_parser::{
    DataType, Definition, Enum, Fields, Module, NamedField, Schema, Struct,
    UnnamedField, Variant,
};

pub(crate) fn compile_schema(Schema { definitions }: &Schema<'_>) -> TokenStream {
    let definintions = definitions.iter().map(compile_definition);

    quote! { #(#definintions)* }
}

fn compile_definition(definition: &Definition<'_>) -> TokenStream {
    let definition = match definition {
        Definition::Module(m) => compile_module(m),
        Definition::Struct(s) => compile_struct(s),
        Definition::Enum(e) => compile_enum(e),
        Definition::TypeAlias(_) | Definition::Const(_) | Definition::Import(_) => quote! {},
    };

    quote! { #definition }
}

fn compile_module(
    Module {
        comment: _,
        name,
        definitions,
    }: &Module<'_>,
) -> TokenStream {
    let name = Ident::new(name, Span::call_site());
    let definitions = definitions.iter().map(compile_definition);

    quote! {
        pub mod #name {
            #(#definitions)*
        }
    }
}

fn compile_struct(
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

                    quote! { ::stef::write_field(w, #id, |w| { #ty }); }
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

                    quote! { ::stef::write_field(w, #id, |w| { #ty }); }
                });

            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

fn compile_enum(
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
                    ::stef::write_id(w, #id);
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
                    ::stef::write_id(w, #id);
                    #fields_body
                }
            }
        }
        Fields::Unit => quote! {
            Self::#name => {
                ::stef::write_id(w, #id);
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

                    quote! { ::stef::write_field(w, #id, |w| { #ty }); }
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

                    quote! { ::stef::write_field(w, #id, |w| { #ty }); }
                });

            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

#[allow(clippy::needless_pass_by_value)]
fn compile_data_type(ty: &DataType<'_>, name: TokenStream) -> TokenStream {
    match ty {
        DataType::Bool => quote! { ::stef::encode_bool(w, #name) },
        DataType::U8 => quote! { ::stef::encode_u8(w, #name) },
        DataType::U16 => quote! { ::stef::encode_u16(w, #name) },
        DataType::U32 => quote! { ::stef::encode_u32(w, #name) },
        DataType::U64 => quote! { ::stef::encode_u64(w, #name) },
        DataType::U128 => quote! { ::stef::encode_u128(w, #name) },
        DataType::I8 => quote! { ::stef::encode_i8(w, #name) },
        DataType::I16 => quote! { ::stef::encode_i16(w, #name) },
        DataType::I32 => quote! { ::stef::encode_i32(w, #name) },
        DataType::I64 => quote! { ::stef::encode_i64(w, #name) },
        DataType::I128 => quote! { ::stef::encode_i128(w, #name) },
        DataType::F32 => quote! { ::stef::encode_f32(w, #name) },
        DataType::F64 => quote! { ::stef::encode_f64(w, #name) },
        DataType::String | DataType::StringRef => quote! { ::stef::encode_string(w, &#name) },
        DataType::Bytes | DataType::BytesRef => quote! { ::stef::encode_bytes(w, &#name) },
        DataType::Vec(_ty) => quote! { ::stef::encode_vec(w, &#name) },
        DataType::HashMap(_kv) => quote! { ::stef::encode_hash_map(w, #name) },
        DataType::HashSet(_ty) => quote! { ::stef::encode_hash_set(w, #name) },
        DataType::Option(_ty) => quote! { ::stef::encode_option(w, #name) },
        DataType::BoxString => quote! { ::stef::encode_string(w, &*#name) },
        DataType::BoxBytes => quote! { ::stef::encode_bytes(w, &*#name) },
        DataType::Tuple(types) => match types.len() {
            size @ 1..=12 => {
                let fn_name = Ident::new(&format!("write_tuple{size}"), Span::call_site());
                quote! { ::stef::#fn_name(w, &#name) }
            }
            0 => panic!("tuple with zero elements"),
            _ => panic!("tuple with more than 12 elements"),
        },
        DataType::Array(_ty, _size) => {
            quote! { ::stef::encode_array(w, &#name) }
        }
        DataType::NonZero(_) | DataType::External(_) => {
            quote! { #name.encode(w) }
        }
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
}
