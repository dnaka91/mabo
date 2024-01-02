# Strongly Typed Encoding Format

Data format and schema, with a type system as strong as Rust's.

Mabo is a data encoding format, that borrows a lot from existing formats like [Protobuf](https://protobuf.dev), [Cap'n Proto](https://capnproto.org) and [Flatbuffers](https://flatbuffers.dev), but in contrast vastly extends the type system to make it as strong as Rust's.

## Why a stronger type system?

Firs and foremost, I personally really enjoy the Rust programming language and its strict but flexible type system.

In the many years that I have used Protobufs, they always have disappointed with the few supported types. Most of the time that resulted in an additional round of validation after decoding the format, to ensure all data is in a valid form.

Many of these validations could be avoided by a stronger type system, which would rule out many wrong states by refusing them upfront.

By extending the type system, data structures can be defined in a way, that ensures the data is already in a valid state after decoding.

## Why not use an existing data format?

### Protobuf

### Cap'n Proto
