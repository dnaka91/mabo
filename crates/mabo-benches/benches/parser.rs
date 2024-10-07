#![expect(missing_docs)]

use criterion::{
    BenchmarkId, Criterion, black_box, criterion_group, criterion_main, profiler::Profiler,
};
use indoc::indoc;
use pprof::{
    criterion::{Output, PProfProfiler},
    flamegraph::{Direction, Options, Palette, color::MultiPalette},
};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn basic(c: &mut Criterion) {
    let input = indoc! {r#"
            use other::one::Type1;
            use other::two;

            const VALUE1: u32 = 100;
            const VALUE2: &string = "hello, world!";

            /// Some sample
            #[deprecated = "outdated"]
            struct SampleStruct<T1, T2> {
                field1: bool @1,
                field2: u32 @2,
                field3: hash_map<u32, vec<string>> @3,
                field4: T1 @4,
                field5: T2 @5,
                /// Comment on a field
                field6: Type1 @6,
                /// Another field comment
                field7: two::Type2<u32, bool, bytes> @7,
            }

            enum Gender {
                Male @1,
                Female @2,
                Other(hash_map<u32, vec<string>> @1) @3,
            }

            /// Redefined type with fixed types
            type SampleTyped = SampleStruct<bool, string>;
        "#};

    mabo_parser::Schema::parse(input, None).unwrap();

    c.bench_function("basic", |b| {
        b.iter(|| mabo_parser::Schema::parse(black_box(input), None));
    });
}

fn large_schema(c: &mut Criterion) {
    let mut g = c.benchmark_group("large_schema");
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let schema = mabo_benches::generate_schema(n);
            mabo_parser::Schema::parse(&schema, None).unwrap();

            b.iter(|| mabo_parser::Schema::parse(black_box(&schema), None));
        });
    }

    g.finish();
}

fn print(c: &mut Criterion) {
    let mut g = c.benchmark_group("print");
    for n in [1, 10, 100, 1000] {
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let schema = mabo_benches::generate_schema(n);
            let schema = mabo_parser::Schema::parse(&schema, None).unwrap();

            b.iter(|| black_box(&schema).to_string());
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
    targets = basic, large_schema, print
);
criterion_main!(benches);
