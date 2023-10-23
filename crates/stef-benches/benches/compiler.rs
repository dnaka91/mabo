use divan::{black_box, Bencher};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    divan::main();
}

#[divan::bench(consts = [1, 10, 100, 1000])]
fn large_schema<const N: usize>(bencher: Bencher) {
    let schema = stef_benches::generate_schema(N);
    let schema = stef_parser::Schema::parse(&schema).unwrap();
    stef_compiler::validate_schema(&schema).unwrap();

    bencher.bench(|| stef_compiler::validate_schema(black_box(&schema)))
}
