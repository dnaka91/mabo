# Schema

Schemas are an essential part of STEF. They define the structure of the data, and thus, how to en- and decode to or from the raw bytes.

<!-- toc -->

## Type mapping

The following describes all the built-in types of the language, together with the equivalent type in each of the supported programming languages.

When generating source code, these are the types that are used.

### Basic types

Basic types are the most primitive, like boolean, integers, strings and raw binary data. The are distinct and don't carry any special behavior.

Some of these are not natively supported in each language, turning into a common type. For example the different integer types are all `number` in TypeScript or `int` in Python.

| Schema  | Rust     | Go        | Kotlin       | TypeScript | Python |
| ------- | -------- | --------- | ------------ | ---------- | ------ |
| bool    | bool     | bool      | Boolean      | boolean    | bool   |
| u8      | u8       | uint8     | UByte        | number     | int    |
| u16     | u16      | uint16    | UShort       | number     | int    |
| u32     | u32      | uint32    | UInt         | number     | int    |
| u64     | u64      | uint64    | ULong        | bigint     | int    |
| u128    | u128     | [big.Int] | [BigInteger] | bigint     | int    |
| i8      | i8       | int8      | Byte         | number     | int    |
| i16     | i16      | int16     | Short        | number     | int    |
| i32     | i32      | int32     | Int          | number     | int    |
| i64     | i64      | int64     | Long         | bigint     | int    |
| i128    | i128     | [big.Int] | [BigInteger] | bigint     | int    |
| f32     | f32      | float32   | Float        | number     | float  |
| f64     | f64      | float64   | Double       | number     | float  |
| string  | String   | string    | String       | string     | str    |
| &string | &str     | string    | String       | string     | str    |
| bytes   | Vec\<u8> | []byte    | ByteArray    | Uint8Array | bytes  |
| &bytes  | &\[u8]   | []byte    | ByteArray    | Uint8Array | bytes  |

[big.Int]: https://pkg.go.dev/math/big#Int
[BigInteger]: https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/math/BigInteger.html

### Generics

Generic types have one or more type parameters. That means they are not bound to a single type, but can be used together with any other type.

For example, a vector `vec<T>` (list of values) can be used as `vec<u32>` to create a list of 32-bit unsigned integers, or as `vec<string>` to crate a list of text values.

#### Vectors `vec<T>`

Containers for multiple values of a single type.

| Language   | Definition |
| ---------- | ---------- |
| Rust       | Vec\<T>    |
| Go         | []T        |
| Kotlin     | List\<T>   |
| TypeScript | T\[]       |
| Python     | list\[T]   |

#### Hash maps `hash_map<K, V>`

Mapping from keys to values, also called dictionaries in some languages. The key must be unique in the map and inserting a new value with the same key will replace the old value.

| Language   | Definition     |
| ---------- | -------------- |
| Rust       | HashMap\<K, V> |
| Go         | map\[K]V       |
| Kotlin     | Map\<K, V>     |
| TypeScript | Map\<K, V>     |
| Python     | dict\[K, V]    |

#### Hash sets `hash_set<T>`

Collection of distinct values. These are basically like a hash map without an associated value, and can be used to ensure that each contained value is only present once.

| Language   | Definition      |
| ---------- | --------------- |
| Rust       | HashSet\<T>     |
| Go         | map\[T]struct{} |
| Kotlin     | Set\<T>         |
| TypeScript | Set\<T>         |
| Python     | set\[T]         |

#### Optionals `option<T>`

Optional values may be present or missing. By default each value must be present in the wire format and this type allows to declare them as potentially absent.

| Language   | Definition     |
| ---------- | -------------- |
| Rust       | Option\<T>     |
| Go         | \*T            |
| Kotlin     | T?             |
| TypeScript | T \| undefined |
| Python     | T \| None      |

### Extended types

These types have special meanings that either allow for more compact representation on the wire format, or are beneficial in low-level languages that give fine grained control over memory.

#### Non-zero values `non_zero<T>`

This wrapper type defines, that the contained type is not zero. Depending on the type this can have a different meaning.

- `u8`-`u128`, `i8`-`i128`: The integer is guaranteed to be non-zero.
- `string`, `bytes`: The value is guaranteed to contain at least one character or byte.

The reason for this type is two-fold. First off, it allows to be more strict about certain values, where a zero number or empty string is not allowed.

Then, on the wire format when combined with an `option<T>`, it allows to use use less bytes for the value. As the value can't take be zero or empty, this state can be used to represent the missing state of the option.

| Language   | Definition  |
| ---------- | ----------- |
| Rust       | NonZero\<T> |
| Go         | T           |
| Kotlin     | T           |
| TypeScript | T           |
| Python     | T           |

#### Boxed strings `box<string>`

Specialized type, that currently only has a specific meaning for Rust. It describes a string value that lives on the heap and is immutable (in contrast to a `string` which can be mutated).

As strings are immutable in most other languages, it's equivalent to the `string` type for these.

| Language   | Definition |
| ---------- | ---------- |
| Rust       | Box\<str>  |
| Go         | string     |
| Kotlin     | String     |
| TypeScript | string     |
| Python     | str        |

#### Boxed bytes `box<bytes>`

Specialized type, that describes an immutable byte array, and is specific to Rust.

Byte arrays are mutable in other languages as well, but they don't have a reasonable way of defining those as immutable.

| Language   | Definition  |
| ---------- | ----------- |
| Rust       | Box\<\[u8]> |
| Go         | \[]byte     |
| Kotlin     | ByteArray   |
| TypeScript | Uint8Array  |
| Python     | bytes       |

## Identifiers

Identifier are an integral part of schemas and are attached to named and unnamed fields inside a struct or enum.

As the wire format doesn't contain any field names, fields have to be identified in some way. This is done by identifiers, which are [varint](../wire-format/index.md#varint-encoding) encoded integers.

## Naming

| Item            | Convention             |
| --------------- | ---------------------- |
| Modules         | `snake_case`           |
| Structs         | `UpperCamelCase`       |
| Struct fields   | `snake_case`           |
| Enums           | `UpperCamelCase`       |
| Enum variants   | `UpperCamelCase`       |
| Constants       | `SCREAMING_SNAKE_CASE` |
| Type parameters | `UpperCamelCase`       |
| Type aliases    | `UpperCamelCase`       |
