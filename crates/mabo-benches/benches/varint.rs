#![expect(missing_docs)]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use mabo_benches::varint;

fn unsigned(c: &mut Criterion) {
    let mut g = c.benchmark_group("unsigned/bincode");
    for value in [
        1,
        u8::MAX.into(),
        u16::MAX.into(),
        u32::MAX.into(),
        u64::MAX.into(),
        u128::MAX,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::bincode::encode_u128(black_box(*value), black_box(&mut buf));
                varint::bincode::decode_u128(black_box(&buf))
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("unsigned/leb128");
    for value in [
        1,
        u8::MAX.into(),
        u16::MAX.into(),
        u32::MAX.into(),
        u64::MAX.into(),
        u128::MAX,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::postcard::encode(black_box(*value), black_box(&mut buf));
                varint::postcard::decode(black_box(&buf))
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("unsigned/vu128");
    for value in [
        1,
        u8::MAX.into(),
        u16::MAX.into(),
        u32::MAX.into(),
        u64::MAX.into(),
        u128::MAX,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::vu128::encode_u128(black_box(*value), black_box(&mut buf));
                varint::vu128::decode_u128(black_box(&buf))
            });
        });
    }
    g.finish();
}

fn signed(c: &mut Criterion) {
    let mut g = c.benchmark_group("signed/bincode");
    for value in [
        -1,
        i8::MIN.into(),
        i16::MIN.into(),
        i32::MIN.into(),
        i64::MIN.into(),
        i128::MIN,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::bincode::encode_i128(black_box(*value), black_box(&mut buf));
                varint::bincode::decode_i128(black_box(&buf))
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("signed/leb128");
    for value in [
        -1,
        i8::MIN.into(),
        i16::MIN.into(),
        i32::MIN.into(),
        i64::MIN.into(),
        i128::MIN,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::postcard::encode_i128(black_box(*value), black_box(&mut buf));
                varint::postcard::decode_i128(black_box(&buf))
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("signed/vu128");
    for value in [
        -1,
        i8::MIN.into(),
        i16::MIN.into(),
        i32::MIN.into(),
        i64::MIN.into(),
        i128::MIN,
    ] {
        let mut buf = [0; 32];
        g.bench_with_input(BenchmarkId::from_parameter(value), &value, |b, value| {
            b.iter(|| {
                varint::vu128::encode_i128(black_box(*value), black_box(&mut buf));
                varint::vu128::decode_i128(black_box(&buf))
            });
        });
    }
    g.finish();
}

criterion_group!(benches, unsigned, signed);
criterion_main!(benches);
