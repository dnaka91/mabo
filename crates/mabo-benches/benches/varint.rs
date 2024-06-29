#![allow(missing_docs)]

use divan::black_box;
use mabo_benches::varint;

fn main() {
    divan::main();
}

trait Unsigned {
    fn run(value: u128, buf: &mut [u8]) -> u128;
}

trait Signed {
    fn run(value: i128, buf: &mut [u8]) -> i128;
}

struct Leb128;

impl Unsigned for Leb128 {
    fn run(value: u128, buf: &mut [u8]) -> u128 {
        varint::postcard::encode(value, buf);
        varint::postcard::decode(buf)
    }
}

impl Signed for Leb128 {
    fn run(value: i128, buf: &mut [u8]) -> i128 {
        varint::postcard::encode_i128(value, buf);
        varint::postcard::decode_i128(buf)
    }
}

struct Bincode;

impl Unsigned for Bincode {
    fn run(value: u128, buf: &mut [u8]) -> u128 {
        varint::bincode::encode_u128(value, buf);
        varint::bincode::decode_u128(buf)
    }
}

impl Signed for Bincode {
    fn run(value: i128, buf: &mut [u8]) -> i128 {
        varint::bincode::encode_i128(value, buf);
        varint::bincode::decode_i128(buf)
    }
}

struct Vu128;

impl Unsigned for Vu128 {
    fn run(value: u128, buf: &mut [u8]) -> u128 {
        varint::vu128::encode_u128(value, buf);
        varint::vu128::decode_u128(buf)
    }
}

impl Signed for Vu128 {
    fn run(value: i128, buf: &mut [u8]) -> i128 {
        varint::vu128::encode_i128(value, buf);
        varint::vu128::decode_i128(buf)
    }
}

#[divan::bench(
    types = [Bincode, Leb128, Vu128],
    args = [
        1,
        u8::MAX.into(),
        u16::MAX.into(),
        u32::MAX.into(),
        u64::MAX.into(),
        u128::MAX,
    ],
)]
fn unsigned<T: Unsigned>(n: u128) -> u128 {
    let mut buf = [0; 32];
    T::run(n, black_box(&mut buf))
}

#[divan::bench(
    types = [Bincode, Leb128, Vu128],
    args = [
        -1,
        i8::MIN.into(),
        i16::MIN.into(),
        i32::MIN.into(),
        i64::MIN.into(),
        i128::MIN,
    ],
)]
fn signed<T: Signed>(n: i128) -> i128 {
    let mut buf = [0; 32];
    T::run(n, black_box(&mut buf))
}
