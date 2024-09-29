#![expect(missing_docs)]

use divan::{black_box, Bencher};
use indoc::indoc;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    divan::main();
}

#[divan::bench]
fn basic(bencher: Bencher<'_, '_>) {
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

    bencher.bench(|| mabo_parser::Schema::parse(black_box(input), None));
}

#[divan::bench(args = [1, 10, 100, 1000])]
fn large_schema(bencher: Bencher<'_, '_>, n: usize) {
    let schema = mabo_benches::generate_schema(n);
    mabo_parser::Schema::parse(&schema, None).unwrap();

    bencher.bench(|| mabo_parser::Schema::parse(black_box(&schema), None));
}

#[divan::bench(args = [1, 10, 100, 1000])]
fn print(bencher: Bencher<'_, '_>, n: usize) {
    let schema = mabo_benches::generate_schema(n);
    let schema = mabo_parser::Schema::parse(&schema, None).unwrap();

    bencher.bench(|| black_box(&schema).to_string());
}
