# Changelog

All notable changes to this project will be documented in this file.

<!-- markdownlint-disable no-duplicate-header -->
<!-- markdownlint-disable no-trailing-spaces -->

## Unreleased

### ⛰️ Features

- Implement encoding in the Rust codegen ([2787b51](https://github.com/dnaka91/stef/commit/2787b51c04803311bf5ca3160b37e7db31d5a8ea))
- Simplify encoding logic ([e8205bc](https://github.com/dnaka91/stef/commit/e8205bcb6749fce6dd1f56ede38128076820bffd))
- Implement basic decoding logic ([f67c572](https://github.com/dnaka91/stef/commit/f67c57220ac2c57961bf54f7f47bca467d3fb20b))
- Skip any decoding for unit structs/variants ([8f5a4ec](https://github.com/dnaka91/stef/commit/8f5a4ec3cd7377ac2f0eb7183c201fbf388d9ce2))
- Create playground crate and fix issues ([b55862f](https://github.com/dnaka91/stef/commit/b55862f3dafe8276faad9309cebff2cdcaec8ea3))
- Extend syntax highlighting ([0d83290](https://github.com/dnaka91/stef/commit/0d83290e1367edc1a8f9c7686bb2032067c177fa))
- Adjust encoding of option<T> ([39c6ccb](https://github.com/dnaka91/stef/commit/39c6ccba84c71688346ce9cd33bc2530ee0a746c))
- Unroll container type en-/decoding ([3d64436](https://github.com/dnaka91/stef/commit/3d64436501d5e395ba27e9acc51bd82aa312a8d8))
  > Instead of relying on generic implementations for the Rust generated
  > code that en- or decodes the data, these are defined as closures.
  > 
  > The main benefit is that it allows optimized calls that would not be
  > possible through the trait system (for example a fast encoding of
  > `Vec<u8>` instead of the regular `Vec<T>` encoding that would write the
  > bytes one-by-one).
- Implement Decode for non-zero types ([8f7e85e](https://github.com/dnaka91/stef/commit/8f7e85ef5ed68e1cd401927b54ad39ba97441fb4))
- Accept glob patterns for schema input files ([d8f5e43](https://github.com/dnaka91/stef/commit/d8f5e439acc25afb096820b66fcf97bae34d3e91))
- Add non-zero collection types ([e0fc43e](https://github.com/dnaka91/stef/commit/e0fc43ea968a9ca1586642eff937005648eed346))
  > This expands from non-zero integer values to non-zero collections,
  > meaning `non_zero<T>` specializations where `T` is one of `string`,
  > `bytes`, `vec<T>`, `hash_map<K, V>` or `hash_set<T>`.
  > 
  > Any other `non_zero<T>` types are now refused as there is no clear way
  > of defining what non-zero means for other types (or at least not a
  > single _correct_ way).
  > 
  > Also, snapshot tests includes missing adjustments from the last commits
  > that introduced further `#[allow(...)]` attributes in the Rust
  > implementation.
- Validate for unique IDs in structs and enums ([3295f0c](https://github.com/dnaka91/stef/commit/3295f0c4300f839b9cbbd4c6ba035b834e4e6c4f))
  > Add the first "compiler" validation logic, which ensures that each ID:
  > - in named or unnamed fields of a struct or enum variant is unique
  > - in variants of an enum is unique
- Validate field and variant name uniqueness ([9137e20](https://github.com/dnaka91/stef/commit/9137e20e3724ac18232375303a153503b51ea435))
  > Ensure that all fields in a struct or enum variant are uniquely named,
  > as well as all variants within a single enum.
- Nicely report compiler errors ([2737f20](https://github.com/dnaka91/stef/commit/2737f20013cfe0d1fbf6b4a80ef101917183119a))
  > Extend parser structs with spans to generate nice error reports with
  > annotated source code, like being done for parser errors.
- Verify that generics are unique and used ([f3e0ced](https://github.com/dnaka91/stef/commit/f3e0ced34080afba025410ef340b4521514b8911))
  > All defined generics in structs or enums must have a unique name and be
  > used in some way inside the definition of the element. These new
  > verifications in the compiler ensure the rules are upheld.
- Improve error output on compile error ([bd15eba](https://github.com/dnaka91/stef/commit/bd15eba414df2b21bce00384ea79634a3c27670b))
- Enable glob parsing for CLI path arguments ([b44bc1b](https://github.com/dnaka91/stef/commit/b44bc1b305828589c4b682b426b6cb94a9b583d8))
  > In addition to only accepting a list of files for the `check` and
  > `format` command, the input is now treated as glob pattern.
- Add snapshot tests for compiler errors ([0efc118](https://github.com/dnaka91/stef/commit/0efc118308a3e79ec1edddb833328c80182ea8cb))
  > Verify good compiler error output by creating several snapshot tests.
  > Also, slightly improve some of the messages.
- En-/decode non-zero integers directly ([f165d61](https://github.com/dnaka91/stef/commit/f165d618e91a70ab805a9f23f1139177150db4f4))
  > Instead of driving the encoding and decoding of non-zero integer types
  > through the Encode/Decode trait, use direct function calls which allows
  > for specialized calls with better performance.
- Ensure uniqueness of all module declarations ([d09e182](https://github.com/dnaka91/stef/commit/d09e182b28fd12e2952888c5f3c002abeb7914ec))
  > Verify that all declared elements within a module (including the schema
  > root which is a module itself) have a unique name so they can't collide.
- Resolve all local types in schemas ([2b518ab](https://github.com/dnaka91/stef/commit/2b518ab70920b86ac8a5e9e6fa8aea5579fdd1aa))
  > Check that all external types defined in any struct or enum definition
  > exist within the bounds of the schema and give nice error messages in
  > case a type can't be located.

### 🐛 Bug Fixes

- Don't double wrap optional types in decode ([a6d3d4b](https://github.com/dnaka91/stef/commit/a6d3d4bde28d28acb0afba123949ed7e5cbfeb98))
- Extend playground and correct issues ([ed24491](https://github.com/dnaka91/stef/commit/ed24491a8361574bb295d34aad6fc70ed408777b))
- Missing semicolon in tuple structs ([d616e92](https://github.com/dnaka91/stef/commit/d616e92414072a396e448f7b8cd39607b69fbbbe))
- Adjust for new clippy lints ([8855572](https://github.com/dnaka91/stef/commit/88555726ef9e9dd38ddc907a8fb6dbfd4884040f))
- Compile more schemas and fix errors ([ba90911](https://github.com/dnaka91/stef/commit/ba9091181ca93c8e94cf6638d5d23705f725c14a))
- Create specialized encoding for non-zero types ([39420b8](https://github.com/dnaka91/stef/commit/39420b80ef0a985f96ab77f42abd9dec508f4621))
- Supress warning about dereference operator ([21b2f69](https://github.com/dnaka91/stef/commit/21b2f6996254ed945af4ff5acc249870785b7f68))
- Correctly check aliases and imports unique identifiers ([9c88e32](https://github.com/dnaka91/stef/commit/9c88e3291a169bb237e726e099468fd14f5766c5))
  > Aliases couldn't be checked to a non-optimal struct format in its first
  > version and imports were skipped when then don't have a final type
  > element.

### 📚 Documentation

- Generate more stylish changelog ([5319fb3](https://github.com/dnaka91/stef/commit/5319fb3417a830042e7bc220fe283046923da349))
- Add changelog ([5b2a15c](https://github.com/dnaka91/stef/commit/5b2a15cad70e53c6c39a93c395fbe8f80382ae56))
- Update flatbuffers homepage in the book ([c469e4e](https://github.com/dnaka91/stef/commit/c469e4e966cfb3866d08369f813eb999a4c3032d))
- Update Java links to 21 release ([e151095](https://github.com/dnaka91/stef/commit/e151095fd37e1379070255e4a233d75f999deac3))
- Expand user guide for basic setup ([d44c12d](https://github.com/dnaka91/stef/commit/d44c12d16e32e4518dd3c60547a33ca0a50eb74f))
- Add a few (far) future ideas ([bbdc490](https://github.com/dnaka91/stef/commit/bbdc49023e6f3121d6498bc1043bcbd05c06229c))
  > Outline a few ideas that would be great to have in the future, but
  > require a significant amount of work. Therefore, these have no time
  > frame attached to them.

### ⚡ Performance

- Extend the benchmark for better resolving checks ([1b7f52a](https://github.com/dnaka91/stef/commit/1b7f52a7b87579f8f48043ff69e278737d15bfea))
  > The current schema generated for benchmarks on large schemas didn't
  > generate any definitions that use type references. Therefore, the
  > benchmark didn't give good insight on type resolution timing.

### 🚜 Refactor

- Generate definitions and impls together ([b32bcfd](https://github.com/dnaka91/stef/commit/b32bcfd8630bc445421ce32b784de6601659aade))
- Rename test file ([86536c9](https://github.com/dnaka91/stef/commit/86536c919c26934a439e4ebd8bac631e92941dc7))
- Switch to more lightweight bench crate ([3870a6c](https://github.com/dnaka91/stef/commit/3870a6c0db7dbbf720c11f812d5e0b94b57939c3))
- Move common deps to the workspace root ([584fa3e](https://github.com/dnaka91/stef/commit/584fa3eb866c2fa67fc43b1fd918a2fc5f4b379f))
- Create benchmarks for the compiler ([f31f94e](https://github.com/dnaka91/stef/commit/f31f94e3ce5141461b6d65973e702aca822ad25d))
- Enable more clippy lints and unify them ([3c206de](https://github.com/dnaka91/stef/commit/3c206de825d94ed2559d93fba79ff41f1155a0af))
  > Ensuring consistent code quality through lints within all the crates
  > that are part of this project.
- Transform some tests into snapshot tests ([0f69ed5](https://github.com/dnaka91/stef/commit/0f69ed5bf740b6ac113001153bf4632c9a651ee1))
- Rename stef-cli crates binary to stef ([92f039b](https://github.com/dnaka91/stef/commit/92f039b82d28686d26fdb36d01b7c728f241f9fc))
  > Although the package is named `stef-cli` to not clash with the `stef`
  > library crate, the binary should still be named `stef`.
- Use underscore in schema file names ([b43e779](https://github.com/dnaka91/stef/commit/b43e779167d2010f4cde191620598f8c721f8388))
  > As part of the type resolution, module names are derived from the file
  > name. Thus, the file name must adhere to the module naming rules.
  > 
  > This replaces all dashes `-` with underscores `_` in the schema files
  > used for snapshot testing.
- Simplify transformation of name elements ([2acd7b7](https://github.com/dnaka91/stef/commit/2acd7b740859d2c0ea713b9fa20789e653c75013))
  > Instead of manually constructing the `Name` instances, use the From/Into
  > traits to do so automatically and with less code.
- Simplify compiler checking logic ([b77cfa2](https://github.com/dnaka91/stef/commit/b77cfa2c8c923791356b760ad6ccc9455f2cc756))
  > Mostly replacing `Result<(), E>` with `Option<E>` to avoid several
  > conversions into the `Result` type, An `Option` is simpler because all
  > the checks never produce any value besides a possible error.

### 🧪 Testing

- Add snapshot tests to stef-build ([1313fe9](https://github.com/dnaka91/stef/commit/1313fe9f99cceee8a883791c99e318768e27f801))
- Enable more snapshot tests and fix errors ([85938a4](https://github.com/dnaka91/stef/commit/85938a4a7532d034b7eccbea1643a95a84434954))

### ⚙️ Miscellaneous Tasks

- Initial commit ([5eb2f2b](https://github.com/dnaka91/stef/commit/5eb2f2b9687146363974ea645de22a8441e890a1))
- Update checkout action to v4 ([4d753d8](https://github.com/dnaka91/stef/commit/4d753d8b30ef3ee7d7e463fb2e7f594aee86d8e7))
- Minor code cleanup of unused code ([a624300](https://github.com/dnaka91/stef/commit/a6243007663ddcf1d4a9da09c9b4b6514dab0db6))

<!-- generated by git-cliff -->
