# Generating code

## Rust

First, make sure you followed the [installation](./installation#rust) instructions, to setup the dependencies in your _Cargo.toml_ properly.

The main entry point is the `mabo-build` crate, which you use as build script in your `build.rs` file. A basic setup looks like this, assuming you have a single schema under `src/sample.mabo`:

```rust
fn main() {
    mabo_build::Compiler::default().compile(&["src/sample.mabo"]).unwrap();
}
```

This will take care of reading and parsing the schema files, then generate the Rust code from them. The code is stored in your `target` folder in a folder specifically for build script output.

In your code you then include the generated files with Rust's `include!` macro. The correct folder can be accessed through the `OUT_DIR` environment variable and combined with the right file name to get the correct path.

Continuing on the previous example, the generated could could be included like this:

```rust
// in src/main.rs

mod sample {
    mabo::include!("sample");
}

fn main() {
    println!("Hello, World!");
}
```

The file name is the same as the input schema file name, but with `.rs` as file extension instead of `.mabo`. The schema file `sample.mabo` becomes `sample.rs`.

### Using the code

From that point on, the generated code can be used like regular Rust code. Extending the example a bit, let's say the schema file contained the following:

```mabo
struct Sample {
    value: u32 @1,
}
```

Then we could use the generated struct as follows:

```rust
// Include Mabo's `Encode` trait to get access to the `encode()` method.
use mabo::Encode;

mod sample {
    mabo::include!("sample");
}

fn main() {
    // Let's create an instance of your `Sample` struct.
    let value = sample::Sample {
        value: 5
    };

    // We can print it out like an Rust value that implements `Debug`:
    println!("{value:?}");

    // Here we encode it into the wire format.
    // - byte 1 for the field identifier.
    // - byte 5 for the actual value.
    // - byte 0 to mark the end of the struct.
    assert_eq!(&[1, 5, 0], value.encode());
}
```
