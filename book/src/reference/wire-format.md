# Wire format

[[toc]]

The wire format takes a lot of inspiration from both [bincode](https://github.com/bincode-org/bincode) and [postcard](https://github.com/jamesmunns/postcard). As tags are involved to identify fields, it takes some ideas from [Protobuf](https://protobuf.dev) and [Cap'n Proto](https://capnproto.org) as well.

## Integers

The encoding for all integer types, with 2-bytes or more, is identical, thus the type only dictates limitations on the allowed value range that is valid for it.

The integer types are:

- Unsigned `u16`, `u32`, `u64`, `u128`.
- Signed `i16`, `i32`, `i64`, `i128`.

### Varint encoding

There are different varint encodings, that all work very similarly, but some end up using less bytes for certain value ranges than others. Generally the both encodings found in `postcard` and `bincode` look fitting.

- `postcard` uses the regular encoding as it's described in many articles and websites, as well as used in the Protobuf format. It creates a chain, where the last bit of each byte tells whether the more data follows or the end is reached. Only for very large numbers (towards the limits of `u64` and `u128`) it consumes more bytes.
- `bincode` uses a slightly different approach where the first byte always tells how many bytes follow, instead of each byte carrying a marker bit.

| Integer value | postcard | bincode |
| ------------- | -------- | ------- |
| 1u8           | 1        | 1       |
| u8::MAX       | 2        | 3       |
| u16::MAX      | 3        | 3       |
| u32::MAX      | 5        | 5       |
| u64::MAX      | 10       | 9       |
| u128::MAX     | 19       | 17      |

As the table shows, both `u64::MAX` and `u128::MAX` take up less space in the `bincode` encoding. But in contrast, the `u8::MAX` takes up an additional byte. This gap in `bincode` happens, because the values `251-255` are used as markers to tell whether `2`, `4`, `8` or `16` bytes follow. Thus these values must be encoded differently and take up the same space as `u16::MAX`.

## Floating point numbers

## Strings and bytes

Both strings (`string`, `&string`) and bytes (`bytes`, `&bytes`) use the same encoding. The difference is that strings ensure valid UTF-8, while bytes are arbitrary data.

The encoding is the length in _varint_ encoding, followed by the data byte-by-byte.

### References

The reference versions `&str` and `&[u8]` have identical encoding to their owned versions. The difference lies in the decoding process. Instead of copying the payload out of the raw data stream, they can reference directly into said stream.

Depending on the support of the programming language, this might not be possible. In that case, they have the same behavior as the owned version.

## Tuples and arrays

Both tuples and arrays have a known length as defined in the schema. Therefore, the types are encoded in sequence and can be decoded without any further information like the length.

## Structs

## Enums
