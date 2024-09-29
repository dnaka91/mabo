---
order: 3
---

# Project Files

Each Mabo project is identified by a `Mabo.toml` file, usually located at the root of a project. For example, if this is a Rust project, the file would be located next to the `Cargo.toml`.

Currently the project file is fairly basic, only allowing to define a single project with the schema files that it uses and some metadata.

Future iterations will involve distribution and consumption of schema collections.

The file must contain a single [package](./packages) declaration and at least have a name and list of files to include:

```toml
[package]
name = "my_schemas"
files = ["schemas/**/*.mabo"]
```
