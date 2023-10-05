use stef::{Decode, Encode};

mod sample {
    include!(concat!(env!("OUT_DIR"), "/sample.rs"));
}

fn main() {
    let mut buf = Vec::new();
    let v = sample::Sample {
        a: 5,
        b: true,
        c: ("Test".into(), -2),
    };
    v.encode(&mut buf);
    println!("{buf:?}");

    let v2 = sample::Sample::decode(&mut &*buf).unwrap();
    assert_eq!(v, v2);

    buf.clear();
    let v = sample::Sample2::Unit;
    v.encode(&mut buf);
    println!("{buf:?}");

    let v2 = sample::Sample2::decode(&mut &*buf).unwrap();
    assert_eq!(v, v2);

    buf.clear();
    let v = sample::Sample2::Tuple(7, 8);
    v.encode(&mut buf);
    println!("{buf:?}");

    let v2 = sample::Sample2::decode(&mut &*buf).unwrap();
    assert_eq!(v, v2);

    buf.clear();
    let v = sample::Sample2::Fields {
        name: "this".into(),
        valid: true,
        dates: vec![
            (2023, 1, 1),
            (2023, 10, 5),
            (2023, 12, sample::CHRISTMAS_DAY),
        ],
    };
    v.encode(&mut buf);
    println!("{buf:?}");

    let v2 = sample::Sample2::decode(&mut &*buf).unwrap();
    assert_eq!(v, v2);

    buf.clear();
    let v = sample::SampleGen {
        raw: vec![5, 6, 7, 8],
        array: [9_i16; 4],
        value: 9,
    };
    v.encode(&mut buf);
    println!("{buf:?}");

    let v2 = sample::SampleGen::decode(&mut &*buf).unwrap();
    assert_eq!(v, v2);
}
