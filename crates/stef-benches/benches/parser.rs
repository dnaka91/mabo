use std::{fmt::Write, hint::black_box};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use indoc::indoc;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn parser(c: &mut Criterion) {
    c.bench_function("basic", |b| {
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

        b.iter(|| stef_parser::Schema::parse(black_box(input)))
    });

    for count in [1, 10, 100, 1000] {
        let schema = generate_schema(count);
        stef_parser::Schema::parse(&schema).unwrap();

        c.bench_with_input(BenchmarkId::new("large_schema", count), &count, |b, _| {
            b.iter(|| stef_parser::Schema::parse(black_box(&schema)))
        });
    }

    for count in [1, 10, 100, 1000] {
        let schema = generate_schema(count);
        let schema = stef_parser::Schema::parse(&schema).unwrap();

        c.bench_with_input(BenchmarkId::new("print", count), &count, |b, _| {
            b.iter(|| black_box(&schema).to_string())
        });
    }
}

fn generate_schema(count: usize) -> String {
    let mut input = String::new();

    for i in 1..=count {
        writeln!(&mut input, "use some::other::module{i};").unwrap();
    }

    input.push('\n');

    for i in 1..=count {
        writeln!(
            &mut input,
            "const VALUE_BOOL_{i}: bool = {};",
            if i % 2 == 0 { "true" } else { "false" }
        )
        .unwrap();
        writeln!(&mut input, "const VALUE_INT_{i}: u32 = {i};").unwrap();
        writeln!(
            &mut input,
            "const VALUE_FLOAT_{i}: f64 = {};",
            i as f64 / 19.0
        )
        .unwrap();
        writeln!(&mut input, "const VALUE_STR_{i}: &string = \"{i}\";").unwrap();
        writeln!(&mut input, "const VALUE_BYTES_{i}: &bytes = [{}];", i % 256).unwrap();
    }

    input.push('\n');

    for i in 1..=count {
        writeln!(&mut input, "/// Some comment {i}").unwrap();
    }

    for i in 1..=count {
        writeln!(&mut input, "#[unit_attribute_{i}]").unwrap();
        writeln!(&mut input, "#[single_value_{i} = \"value_{i}\"]").unwrap();
        writeln!(
            &mut input,
            "#[multi_value_{i}(value_a = true, value_b(test1, test2), value_c)]"
        )
        .unwrap();
    }

    input.push_str("struct SampleNamed {\n");
    for i in 1..=count {
        writeln!(&mut input, "    field_str_{i:05}: string @{i},").unwrap();
        writeln!(
            &mut input,
            "    field_gen_{i:05}: vec<hash_map<u32, (bool, string, option<f64>)>> @{},",
            i + count
        )
        .unwrap();
    }
    input.push_str("}\n");

    input.push_str("\nstruct SampleUnnamed(");
    for i in 1..=count {
        write!(&mut input, "string @{i},").unwrap();
        write!(
            &mut input,
            "vec<hash_map<u32, (bool, string, option<f64>)>> @{},",
            i + count
        )
        .unwrap();
    }
    input.push_str(")\n");

    input.push_str("\nenum SampleEnum {\n");
    for i in 1..=count {
        writeln!(&mut input, "    VariantNamed{i} {{").unwrap();
        writeln!(&mut input, "        field_str: string @1,").unwrap();
        writeln!(&mut input, "        field_gen: vec<hash_set<u32>> @2,").unwrap();
        writeln!(&mut input, "    }} @{i},").unwrap();
        write!(&mut input, "    VariantUnnamed{i}(").unwrap();
        write!(&mut input, "string @1, vec<hash_set<u32>> @2").unwrap();
        writeln!(&mut input, ") @{},", i + count).unwrap();
    }
    input.push_str("}\n");

    input.push('\n');

    for i in 1..=count {
        writeln!(&mut input, "type Alias{i} = SampleNamed;").unwrap();
    }

    input
}

criterion_group!(benches, parser);
criterion_main!(benches);
