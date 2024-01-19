# Generators

Code generators are the part of Mabo that turn the schema into actual source code that can be consumed by projects.

The sub-sections describe details for all the officially supported generators. These are all written in Rust, but these are not limited to one language.

The mabo compiler can output the parsed schemas in [JSON](https://www.json.org/json-en.html), a data format very common and widespread. Therefore, generators can be written in any language, consume the output and turn it into source code.

## How generators work

Each generator is usually paired with a runtime library to make the generated code work. This handles the low-level workings of the format like encoding individual values.

How a generator is invoked depends on the language. For example:

- Go projects call a binary to generate the code and include the output directly in the repository.
- Java projects use a Gradle plugin to generate the data structures on the fly.
- JavaScript and TypeScript projects use a bundler plugin to generate the data structures.
- Rust projects use a `build.rs` build script file to generate the data structures.
