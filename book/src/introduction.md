# Strongly Typed Encoding Format

Data format and schema, with a type system as strong as Rust's.

<div style="display: flex; justify-content: center;">
    <svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 24 24" width="128" height="128" >
        <path d="m0 0h24v24h-24z" fill="#222034" />
        <g fill="#3f3f74">
            <path d="m1 1h10v2h-8v2h8v6h-10v-2h8v-2h-8z"/>
            <path d="m13 1h10v2h-4v8h-2v-8h-4z"/>
            <path d="m1 13h10v2h-8v2h4v2h-4v2h8v2h-10z"/>
            <path d="m13 13h10v2h-8v2h4v2h-4v4h-2z"/>
        </g>
        <g fill="#5b6ee1">
            <path d="m3 5h7v5h-9v-1h8v-3h-6zm-2-4h10v1h-9v5h-1z"/>
            <path d="m17 3h1v8h-1zm-4-2h10v1h-9v1h-1z"/>
            <path d="m3 21h8v1h-8zm-0-4h4v1h-4zm-2-4h10v1h-9v9h-1z"/>
            <path d="m15 17h4v1h-4zm-2-4h10v1h-9v9h-1z"/>
        </g>
    </svg>
</div>

As the name suggests, STEF is a data encoding format, that borrows a lot from existing formats like [Protobuf](https://protobuf.dev), [Cap'n Proto](https://capnproto.org) and [Flatbuffers](https://flatbuffers.dev), but in contrast vastly extends the type system to make it as strong as Rust's.

## Why a stronger type system?

Firs and foremost, I personally really enjoy the Rust programming language and its strict but flexible type system.

In the many years that I have used Protobufs, they always have disappointed with the few supported types. Most of the time that resulted in an additional round of validation after decoding the format, to ensure all data is in a valid form.

Many of these validations could be avoided by a stronger type system, which would rule out many wrong states by refusing them upfront.

By extending the type system, data structures can be defined in a way, that ensures the data is already in a valid state after decoding.

## Why not use an existing data format?

### Protobuf

### Cap'n Proto
