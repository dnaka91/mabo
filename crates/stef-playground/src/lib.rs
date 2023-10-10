mod sample {
    include!(concat!(env!("OUT_DIR"), "/sample.rs"));
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use stef::{Decode, Encode};

    use super::sample;

    fn roundtrip<T: Debug + PartialEq + Decode + Encode>(value: T) {
        let mut buf = Vec::new();
        value.encode(&mut buf);
        println!("{}: {buf:?}", std::any::type_name::<T>());

        let value2 = T::decode(&mut &*buf).unwrap();
        assert_eq!(value, value2);
    }

    #[test]
    fn sample() {
        roundtrip(sample::Sample {
            a: 5,
            b: true,
            c: ("Test".into(), -2),
        });
    }

    #[test]
    fn sample2_unit() {
        roundtrip(sample::Sample2::Unit);
    }

    #[test]
    fn sample2_tuple() {
        roundtrip(sample::Sample2::Tuple(7, 8));
    }

    #[test]
    fn sample2_fields() {
        roundtrip(sample::Sample2::Fields {
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
    fn sample_gen() {
        roundtrip(sample::gens::SampleGen {
            raw: vec![5, 6, 7, 8],
            array: [9_i16; 4],
            value: 9,
        });
    }

    #[test]
    fn sample_gen2() {
        roundtrip(sample::gens::SampleGen2::Value(sample::SampleAlias {
            a: 50,
            b: false,
            c: (String::new(), -10),
        }));
    }

    #[test]
    fn specials_options_some() {
        roundtrip(sample::specials::SomeOptions {
            maybe_int: Some(5),
            maybe_text: Some("hi".into()),
            maybe_tuple: Some((20, 30)),
            nested: Some(Some(8)),
            vec_maybe: vec![Some(true), None, Some(false)],
        });
    }

    #[test]
    fn specials_options_none() {
        roundtrip(sample::specials::SomeOptions {
            maybe_int: None,
            maybe_text: None,
            maybe_tuple: None,
            nested: None,
            vec_maybe: vec![None, None],
        });
    }
}
