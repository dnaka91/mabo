#![allow(missing_docs, clippy::missing_errors_doc)]

mod sample {
    stef::include!("sample");
}

mod schemas {
    mod alias_basic {
        stef::include!("alias_basic");
    }

    mod attribute_multi {
        stef::include!("attribute_multi");
    }

    mod attribute_single {
        stef::include!("attribute_single");
    }

    mod attribute_unit {
        stef::include!("attribute_unit");
    }

    mod attributes_min_ws {
        stef::include!("attributes_min_ws");
    }

    mod attributes {
        stef::include!("attributes");
    }

    mod const_basic {
        stef::include!("const_basic");
    }

    mod const_string {
        stef::include!("const_string");
    }

    mod enum_basic {
        stef::include!("enum_basic");
    }

    mod enum_generics {
        stef::include!("enum_generics");
    }

    mod enum_many_ws {
        stef::include!("enum_many_ws");
    }

    mod enum_min_ws {
        stef::include!("enum_min_ws");
    }

    mod import_basic {
        stef::include!("import_basic");
    }

    mod other {
        stef::include!("other");
    }

    mod second {
        stef::include!("second");
    }

    mod module_basic {
        stef::include!("module_basic");
    }

    mod schema_basic {
        stef::include!("schema_basic");
    }

    mod struct_basic {
        stef::include!("struct_basic");
    }

    mod struct_generics {
        stef::include!("struct_generics");
    }

    mod struct_many_ws {
        stef::include!("struct_many_ws");
    }

    mod struct_min_ws {
        stef::include!("struct_min_ws");
    }

    mod struct_tuple {
        stef::include!("struct_tuple");
    }

    mod types_basic {
        stef::include!("types_basic");
    }

    mod types_generic {
        stef::include!("types_generic");
    }

    mod types_nested {
        stef::include!("types_nested");
    }

    mod types_non_zero {
        stef::include!("types_non_zero");
    }

    mod types_ref {
        stef::include!("types_ref");
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use stef::{Decode, Encode};

    use super::sample;

    fn roundtrip<T: Debug + PartialEq + Decode + Encode>(value: &T) {
        let mut buf = Vec::new();
        value.encode(&mut buf);
        println!("{}: {buf:?}", std::any::type_name::<T>());

        let value2 = T::decode(&mut &*buf).unwrap();
        assert_eq!(*value, value2);
    }

    #[test]
    fn sample() {
        roundtrip(&sample::Sample {
            a: 5,
            b: true,
            c: ("Test".into(), -2),
        });
    }

    #[test]
    fn sample2_unit() {
        roundtrip(&sample::Sample2::Unit);
    }

    #[test]
    fn sample2_tuple() {
        roundtrip(&sample::Sample2::Tuple(7, 8));
    }

    #[test]
    fn sample2_fields() {
        roundtrip(&sample::Sample2::Fields {
            name: "this".into(),
            valid: true,
            dates: vec![
                (2023, 1, 1),
                (2023, 10, 5),
                (2023, 12, sample::CHRISTMAS_DAY),
            ],
        });
    }

    #[test]
    fn sample3() {
        roundtrip(&sample::Sample3(true, (vec![1, 2, 3, 4, 5], -500_000)));
    }

    #[test]
    fn sample_gen() {
        roundtrip(&sample::gens::SampleGen {
            raw: vec![5, 6, 7, 8],
            array: [9_i16; 4],
            value: 9,
        });
    }

    #[test]
    fn sample_gen2() {
        roundtrip(&sample::gens::SampleGen2::Value(sample::SampleAlias {
            a: 50,
            b: false,
            c: (String::new(), -10),
        }));
    }

    #[test]
    fn specials_options_some() {
        roundtrip(&sample::specials::SomeOptions {
            maybe_int: Some(5),
            maybe_text: Some("hi".into()),
            maybe_tuple: Some((20, 30)),
            nested: Some(Some(8)),
            vec_maybe: vec![Some(true), None, Some(false)],
        });
    }

    #[test]
    fn specials_options_none() {
        roundtrip(&sample::specials::SomeOptions {
            maybe_int: None,
            maybe_text: None,
            maybe_tuple: None,
            nested: None,
            vec_maybe: vec![None, None],
        });
    }
}
