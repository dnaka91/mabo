use divan::{black_box, Bencher};
use indoc::indoc;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    divan::main();
}

#[divan::bench]
fn basic(bencher: Bencher) {
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

    stef_parser::Schema::parse(input).unwrap();

    bencher.bench(|| stef_parser::Schema::parse(black_box(input)));
}

#[divan::bench(consts = [1, 10, 100, 1000])]
fn large_schema<const N: usize>(bencher: Bencher) {
    let schema = stef_benches::generate_schema(N);
    stef_parser::Schema::parse(&schema).unwrap();

    bencher.bench(|| stef_parser::Schema::parse(black_box(&schema)))
}

#[divan::bench(consts = [1, 10, 100, 1000])]
fn print<const N: usize>(bencher: Bencher) {
    let schema = stef_benches::generate_schema(N);
    let schema = stef_parser::Schema::parse(&schema).unwrap();

    bencher.bench(|| black_box(&schema).to_string())
}
