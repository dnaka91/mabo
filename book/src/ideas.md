# Ideas

The following are general ideas for this project, that are not yet implemented or even close to being developed, but definitely on the list of ideas for the future.

[[toc]]

## Documentation generator

Similar to Rust's [rustdoc](https://doc.rust-lang.org/rustdoc/index.html), it would be nice to have a documentation generator built in. The doc comments can be extended to be parsed as Markdown and allow for rich formatting in the docs.

As a nice side effect, if parsing the comments as Markdown upfront it can be used to transform the docs into native documentation where Markdown is not supported (for example JavaDoc in Java/Kotlin projects).

## Central registry for schemas

Rust has [crates.io](https://crates.io) as central repository for sharing libraries. In similar fashion, a registry for schema files could be created. It would allow easy sharing of schemas.

This should be kept simple and then extended further as features are needed.

Something that is missing from crates.io is to allow for private registries that can be self-hosted, while still being able to get schemas from the main repository (like a proxy, preferring local packages but pulling and caching any schemas that are not present locally).

## Breaking change detection

The schema allows for certain changes that are backwards compatible and do not create a break in the wire format. That means older generated code is still able to read the data despite being created by a newer version of the schema. But the rules are difficult to always keep in mind and mistakes can easily be made.

To avoid accidental breaking changes, it would be nice to have an auto-detection mechanism that compares the current version to a previous one whenever changes are made. The _old_ version could be the current Git HEAD compared to the local changes.

## Schema evolution

When decoding a value, it may contain new fields and enum variants that are not known to the decoder yet. This can happen if the schema changed and but was only updated in one of two parties. For example a server might have introduced new fields to a response, but not all clients have updated to the new schema yet.

The same can happen the other way around. For example, if the data was saved in some form of storage and the schema evolved in the meantime, the decoder might encounter old data that lacks the newer content.

In both cases, the schema must be able to handle missing or unknown fields. Several rules must be upheld when updating a schema, to ensure it is both forward and backward compatible.

### Skip fields without knowing the exact type

This section explains how a decoder is able to process payloads that contain newer or unknown fields, given these were introduced in a backward compatible way.

Without the new schema it's not possible to make decisions about the data that follows after a field identifier. To work around this, reduced information can be encoded into the identifier.

Only a few  details are important for the decoder to proceed, not needing full type information:

- Is the value a variable integer?
  - Skip over individual bytes until the end marker is found
- Is the value length delimited?
  - Parse the delimiter, which is always a _varint_, and skip over the length.
- Is the value a nested struct or enum?
  - Step into the nested type and skip over all its fields.
- Is the value of fixed length?
  - Skip over the fixed length of 1 (`bool`, `u8` and `i8`), 4 (`f32`) or 8 (`f64`) bytes.

Furthermore, this information is only needed for direct elements of a struct or enum variant, as this allows to skip over the whole field. Types nested into another, like a `vec<u32>` for example, don't need to provide this information for each element again.
