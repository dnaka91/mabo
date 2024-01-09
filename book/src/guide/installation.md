# Installation

## Command Line Interface

The CLI tool is not strictly needed to compile and use Mabo schemas, but it contains several helpful elements like a validator for schema correctness, a formatter and helper for setting up Mabo in your project.

### From source

Currently it can only be installed from source, using Cargo:

```sh
cargo install --git https://github.com/dnaka91/mabo.git mabo-cli
```

## Rust

For Rust projects, two crates are needed to work with Mabo schemas and data. One is the `mabo` crate for runtime support, which contains all components that are used by the generated code.

The other one is the `mabo-build` crate which generates the Rust code from schema files. It is used as [Build Script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) usually in the _build.rs_ file of your project.

You can use Cargo's [add](https://doc.rust-lang.org/cargo/commands/cargo-add.html) command to add those dependencies:

```sh
cargo add --git https://github.com/dnaka91/mabo.git         mabo
cargo add --git https://github.com/dnaka91/mabo.git --build mabo-build
```

Or specify them in your _Cargo.toml_ manually:

```toml
[dependencies]
mabo = { git = "https://github.com/dnaka91/mabo.git" }

[build-dependencies]
mabo-build = { git = "https://github.com/dnaka91/mabo.git" }
```
