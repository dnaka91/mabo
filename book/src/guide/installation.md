# Installation

<!-- toc -->

## Command Line Interface

The CLI tool is not strictly needed to compile and use STEF schemas, but it contains several helpful elements like a validator for schema correctness, a formatter and helper for setting up STEF in your project.

### From source

Currently it can only be installed from source, using Cargo:

```sh
cargo install --git https://github.com/dnaka91/stef.git stef-cli
```

## Rust

For Rust projects, two crates are needed to work with STEF schemas and data. One is the `stef` crate for runtime support, which contains all components that are used by the generated code.

The other one is the `stef-build` crate which generates the Rust code from schema files. It is used as [Build Script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) usually in the _build.rs_ file of your project.

You can use Cargo's [add](https://doc.rust-lang.org/cargo/commands/cargo-add.html) command to add those dependencies:

```sh
cargo add --git https://github.com/dnaka91/stef.git         stef
cargo add --git https://github.com/dnaka91/stef.git --build stef-build
```

Or specify them in your _Cargo.toml_ manually:

```toml
[dependencies]
stef = { git = "https://github.com/dnaka91/stef.git" }

[build-dependencies]
stef-build = { git = "https://github.com/dnaka91/stef.git" }
```
