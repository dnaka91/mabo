#![expect(missing_docs)]

use criterion::{
    BenchmarkId, Criterion, black_box, criterion_group, criterion_main, profiler::Profiler,
};
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::{Direction, Options, Palette, color::MultiPalette},
};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn validate_large_schema(c: &mut Criterion) {
    let mut g = c.benchmark_group("validate_large_schema");
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let schema = mabo_benches::generate_schema(n);
            let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
            mabo_compiler::validate_schema(&schema).unwrap();

            b.iter(|| mabo_compiler::validate_schema(black_box(&schema)));
        });
    }

    g.finish();
}

fn resolve_large_schema(c: &mut Criterion) {
    let mut g = c.benchmark_group("resolve_large_schema");
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let schema = mabo_benches::generate_schema(n);
            let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
            mabo_compiler::validate_schema(&schema).unwrap();

            let list = &[("bench", black_box(&schema))];

            b.iter(|| mabo_compiler::resolve_schemas(black_box(list)));
        });
    }

    g.finish();
}

fn simplify_large_schema(c: &mut Criterion) {
    let mut g = c.benchmark_group("simplify_large_schema");
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let schema = mabo_benches::generate_schema(n);
            let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
            let _ = mabo_compiler::simplify_schema(&schema);

            b.iter(|| mabo_compiler::simplify_schema(black_box(&schema)));
        });
    }

    g.finish();
}

fn profiler() -> impl Profiler {
    let mut opts = Options::default();
    opts.colors = Palette::Multi(MultiPalette::Rust);
    opts.direction = Direction::Inverted;

    PProfProfiler::new(100, Output::Flamegraph(Some(opts)))
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(profiler());
    targets = validate_large_schema, resolve_large_schema, simplify_large_schema
);
criterion_main!(benches);
