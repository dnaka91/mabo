#![allow(missing_docs)]
#![expect(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

use std::fmt::Write;

pub mod varint;

#[must_use]
pub fn generate_schema(count: usize) -> String {
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
        writeln!(&mut input, "const VALUE_STR_{i}: &String = \"{i}\";").unwrap();
        writeln!(&mut input, "const VALUE_BYTES_{i}: &Bytes = [{}];", i % 256).unwrap();
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
        writeln!(&mut input, "    field_str_{i:05}: String @{i},").unwrap();
        writeln!(
            &mut input,
            "    field_gen_{i:05}: Vec<HashMap<u32, (bool, String, Option<f64>)>> @{},",
            i + count
        )
        .unwrap();
    }
    input.push_str("}\n");

    input.push_str("\nstruct SampleUnnamed(");
    for i in 1..=count {
        write!(&mut input, "String @{i},").unwrap();
        write!(
            &mut input,
            "Vec<HashMap<u32, (bool, String, Option<f64>)>> @{},",
            i + count
        )
        .unwrap();
    }
    input.push_str(")\n");

    input.push_str("\nenum SampleEnum {\n");
    for i in 1..=count {
        writeln!(&mut input, "    VariantNamed{i} {{").unwrap();
        writeln!(&mut input, "        field_str: String @1,").unwrap();
        writeln!(&mut input, "        field_gen: Vec<HashSet<u32>> @2,").unwrap();
        writeln!(&mut input, "    }} @{i},").unwrap();
        write!(&mut input, "    VariantUnnamed{i}(").unwrap();
        write!(&mut input, "String @1, Vec<HashSet<u32>> @2").unwrap();
        writeln!(&mut input, ") @{},", i + count).unwrap();
    }
    input.push_str("}\n");

    input.push('\n');

    for i in 1..=count {
        writeln!(&mut input, "type Alias{i} = SampleNamed;").unwrap();
    }

    input.push_str("\nmod resolve {\n");
    writeln!(&mut input, "    struct Sample {{").unwrap();
    for i in 1..=count {
        writeln!(&mut input, "        field{i:05}: Type{i:05} @{i},").unwrap();
    }
    writeln!(&mut input, "    }}").unwrap();
    for i in 1..=count {
        writeln!(&mut input, "    struct Type{i:05} {{").unwrap();
        writeln!(&mut input, "        value1: String @1,").unwrap();
        writeln!(&mut input, "        value2: i64 @2,").unwrap();
        writeln!(&mut input, "        value3: u64 @3,").unwrap();
        writeln!(&mut input, "    }}").unwrap();
    }
    input.push_str("}\n");

    input
}
