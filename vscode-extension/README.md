# 🍲 Mabo - Visual Studio Code Extension

Data format and schema, with a type system as strong as Rust's.

This project is currently still in a _pre-alpha_ stage and undergoing many changes until the first alpha release. Nonetheless, it should already be usable from the Rust side but expect breaking changes often.

For further details about how to get started, the schema language or anything else, please consult the official [project website](https://dnaka91.github.io/mabo/).

## Using the extension

Currently this extension doesn't bundle the necessary LSP binary. If you have Rust installed, you can use Cargo to build the LSP:

```sh
cargo install --git https://github.com/dnaka91/mabo.git mabo-lsp
```

Then, make sure the binary is globally accessible by putting the Cargo binary folder to your `$PATH` environment variable. This is usually located at `~/.cargo/bin`.

## License

This project is licensed under [MIT License](LICENSE.md) (or <http://opensource.org/licenses/MIT>).
