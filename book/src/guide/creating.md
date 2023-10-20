# Creating schemas

<!-- toc -->

Basic types

- Signed integers: `i8`, `i16`, `i32`, `i64` and `i128`.
- Unsigned integers: `u8`, `u16`, `u32`, `u64` and `u128`.
- Floating point numbers: `f32` and `f64`.
- Strings: `string`.
- Bytes: `bytes`.

Colletions

- Vectors: `vec<T>`.
- Hash maps: `hash_map<K, V>`.
- Hash sets: `hash_set<T>`.

Special types

- Optionals: `option<T>`.
- Non-zero: `non_zero<T>`.
- Boxed strings: `box<string`.
- Boxed bytes: `box<bytes>`.

```rust,ignore
{{#include creating/basic.stef}}
```
