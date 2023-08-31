# General idea

A data format and schema, very similar to Google's Protobuf, but with the full fledged type system of Rust.

- It should look and feel very close to Rust. Not just the type system, but the way the schema language looks like should resemble it very closely.
- Wire format very close to what `postcard` does, of course with adjustments here and there to include the concepts of Protobuf (like field markers).
- Each language generator should be natively integrated into the language's ecosystem. For example,
for Rust it would be a crate that can generate the code in a `build.rs` file, or for Kotlin it would
be a Gradle plugin.

## Additions and improvements over Protobuf

Things that should definitely be added or improved over Protobuf. Also, keep _cap'n proto_ in mind, it has several similar additions like constants and generics.

- Generics
- Tuples
- Rust enums
- Type aliases
- Full range of integer types and basic Rust types
- Support for borrowed data like `&str` and `&[u8]` that reference into the parsed buffer.
- Support for streamed parsing
- Immutable-ish types like `Box<str>`.
- Shared types like `Rc<str>` / `Arc<str>`, even on the wire level. They'd be put into a global ID to value map and be referenced in the fields. (maybe for later, seems hard to get right?)
- Fields are required by default, marked optional with the `Option<T>` type.
- Rust style attributes for further customization. For example, `#[fixed]` to force an integer field not to be compressed (instead of extra types like `fixed32` in Protobuf).
- Avoid type repetition like all variants of an enum being prefixed with the enum name. Instead, add the prefix in the generator for languages that need it (like C).
- Nesting, exactly like in Rust with modules, but inside of other elements like structs respectively.
- Allow for modules inside a file to further scope and group elements together.
- Constants (maybe?)
- Statics (maybe?)

## Lending from Cap'n Proto

These features are concepts from cap'n proto that are missing from Protobuf, and might be interesting to adopt into the new schema language (and aren't listed in the previous section yet).

- Default values: maybe
- Unions: **NO** (Rust enums solve this)
- Groups: **NO** (Rust enums solve this)
- Interfaces: _probably not_
  - Don't really see the need for this. Not interested in defining interfaces, this is supposed to be a data encoding format after all. Exchanging data, not functionality or contracts.
- Unique IDs: **NO** (never really got the point of those)

## What about gRPC

Would recommend people to simply build their own system on top of QUIC, as gRPC is all HTTP/2 under the hood and not as lightweight as it promises to be.

Eventually as a future plan, could introduce Rust traits that can define services in some form. This might reduce flexibility, but allow to have some convenient thin wrapper over QUIC that reduces boilerplate and can be a solution for simple or common setups.

## Initial language support

- Rust (obviously)
- Go
- Kotlin
- Java (automatically through Kotlin, maybe separate generator later)
- TypeScript
- Python (?)
