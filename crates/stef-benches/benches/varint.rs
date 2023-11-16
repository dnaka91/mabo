#![allow(missing_docs)]

use divan::black_box;
use stef_benches::varint;

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

#[divan::bench(
    types = [Leb128, Bincode],
    consts = [
        1,
        u8::MAX as u128,
        u16::MAX as u128,
        u32::MAX as u128,
        u64::MAX as u128,
        u128::MAX,
    ],
)]
fn unsigned<const N: u128, T: Unsigned>() -> u128 {
    let mut buf = [0; 19];
    T::run(black_box(N), black_box(&mut buf))
}

#[divan::bench(
    types = [Leb128, Bincode],
    consts = [
        -1,
        i8::MIN as i128,
        i16::MIN as i128,
        i32::MIN as i128,
        i64::MIN as i128,
        i128::MIN,
    ],
)]
fn signed<const N: i128, T: Signed>() -> i128 {
    let mut buf = [0; 19];
    T::run(black_box(N), black_box(&mut buf))
}
