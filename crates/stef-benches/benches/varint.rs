use std::hint::black_box;

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use stef_benches::varint;

fn varint(c: &mut Criterion) {
    let mut g = c.benchmark_group("varint");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for i in [
        1,
        u8::MAX.into(),
        u16::MAX.into(),
        u32::MAX.into(),
        u64::MAX.into(),
        u128::MAX,
    ] {
        g.bench_with_input(BenchmarkId::new("leb128", i), &i, |b, i| {
            let mut buf = [0; 19];
            let value = *i;
            b.iter(|| {
                varint::postcard::encode(black_box(value), black_box(&mut buf));
                varint::postcard::decode(black_box(&buf))
            });
        });
        g.bench_with_input(BenchmarkId::new("bincode", i), &i, |b, i| {
            let mut buf = [0; 19];
            let value = *i;
            b.iter(|| {
                varint::bincode::encode_u128(black_box(value), black_box(&mut buf));
                varint::bincode::decode_u128(black_box(&buf))
            });
        });
    }
    g.finish();

    let mut g = c.benchmark_group("varint_signed");
    g.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for i in [
        -1,
        i8::MIN.into(),
        i16::MIN.into(),
        i32::MIN.into(),
        i64::MIN.into(),
        i128::MIN,
    ] {
        g.bench_with_input(BenchmarkId::new("leb128", i), &i, |b, i| {
            let mut buf = [0; 19];
            let value = *i;
            b.iter(|| {
                varint::postcard::encode_i128(black_box(value), black_box(&mut buf));
                varint::postcard::decode_i128(black_box(&buf))
            });
        });
        g.bench_with_input(BenchmarkId::new("bincode", i), &i, |b, i| {
            let mut buf = [0; 19];
            let value = *i;
            b.iter(|| {
                varint::bincode::encode_i128(black_box(value), black_box(&mut buf));
                varint::bincode::decode_i128(black_box(&buf))
            });
        });
    }
    g.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().plotting_backend(criterion::PlottingBackend::Plotters);
    targets = varint
}
criterion_main!(benches);
