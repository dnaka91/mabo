# Ideas

The following are general ideas for this project, that are not yet implemented or even close to being developed, but definitely on the list of ideas for the future.

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
