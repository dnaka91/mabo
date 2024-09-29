#![allow(missing_docs)]

mod evolution {
    mabo::include!("evolution");
}

mod sample {
    mabo::include!("sample");
}

mod schemas {
    mod alias_basic {
        mabo::include!("alias_basic");
    }

    mod attribute_multi {
        mabo::include!("attribute_multi");
    }

    mod attribute_single {
        mabo::include!("attribute_single");
    }

    mod attribute_unit {
        mabo::include!("attribute_unit");
    }

    mod attributes_min_ws {
        mabo::include!("attributes_min_ws");
    }

    mod attributes {
        mabo::include!("attributes");
    }

    mod const_basic {
        mabo::include!("const_basic");
    }

    mod const_string {
        mabo::include!("const_string");
    }

    mod enum_basic {
        mabo::include!("enum_basic");
    }

    mod enum_generics {
        mabo::include!("enum_generics");
    }

    mod enum_many_ws {
        mabo::include!("enum_many_ws");
    }

    mod enum_min_ws {
        mabo::include!("enum_min_ws");
    }

    mod import_basic {
        mabo::include!("import_basic");
    }

    mod mixed {
        mabo::include!("mixed");
    }

    mod other {
        mabo::include!("other");
    }

    mod second {
        mabo::include!("second");
    }

    mod module_basic {
        mabo::include!("module_basic");
    }

    mod schema_basic {
        mabo::include!("schema_basic");
    }

    mod struct_basic {
        mabo::include!("struct_basic");
    }

    mod struct_generics {
        mabo::include!("struct_generics");
    }

    mod struct_many_ws {
        mabo::include!("struct_many_ws");
    }

    mod struct_min_ws {
        mabo::include!("struct_min_ws");
    }

    mod struct_tuple {
        mabo::include!("struct_tuple");
    }

    mod types_basic {
        mabo::include!("types_basic");
    }

    mod types_generic {
        mabo::include!("types_generic");
    }

    mod types_nested {
        mabo::include!("types_nested");
    }

    mod types_non_zero {
        mabo::include!("types_non_zero");
    }

    mod types_ref {
        mabo::include!("types_ref");
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use mabo::{Decode, Encode};

    use super::{evolution, sample};

    fn roundtrip<T: Debug + PartialEq + Decode + Encode>(value: &T) {
        let mut buf = Vec::new();
        value.encode(&mut buf);
        println!("{}: {buf:?}", std::any::type_name::<T>());

        let value2 = T::decode(&mut &*buf).unwrap();
        assert_eq!(*value, value2);
    }

    #[test]
    fn evolution() {
        let mut buf = Vec::new();
        evolution::Version2 {
            field1: 5,
            field2: "Test".to_owned(),
        }
        .encode(&mut buf);

        println!("{}: {buf:?}", std::any::type_name::<evolution::Version2>());

        let value = evolution::Version1::decode(&mut &*buf).unwrap();
        assert_eq!(5, value.field1);
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
