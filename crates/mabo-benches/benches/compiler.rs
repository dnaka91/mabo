#![expect(missing_docs)]

use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    divan::main();
}

#[divan::bench(args = [1, 10, 100, 1000])]
fn validate_large_schema(bencher: Bencher<'_, '_>, n: usize) {
    let schema = mabo_benches::generate_schema(n);
    let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
    mabo_compiler::validate_schema(&schema).unwrap();

    bencher.bench(|| mabo_compiler::validate_schema(black_box(&schema)));
}

#[divan::bench(args = [1, 10, 100, 1000])]
fn resolve_large_schema(bencher: Bencher<'_, '_>, n: usize) {
    let schema = mabo_benches::generate_schema(n);
    let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
    mabo_compiler::validate_schema(&schema).unwrap();

    let list = &[("bench", black_box(&schema))];

    bencher.bench(|| mabo_compiler::resolve_schemas(black_box(list)));
}

#[divan::bench(args = [1, 10, 100, 1000])]
fn simplify_large_schema(bencher: Bencher<'_, '_>, n: usize) {
    let schema = mabo_benches::generate_schema(n);
    let schema = mabo_parser::Schema::parse(&schema, None).unwrap();
    let _ = mabo_compiler::simplify_schema(&schema);

    bencher.bench(|| mabo_compiler::simplify_schema(black_box(&schema)));
}
