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

## Identifiers

Identifiers are an essential part of the format. They mark the start of a field or enum variant and describe which one it is, so the decoder knows how to parse the following data and assign it to the right element of a struct or enum.

These IDs are regular **32-bit unsigned integers**, and may encode additional information together with field or variant number.

They are encoded exactly the same as regular integers, with **Varint** encoding.

::: tip
Due to how **Varint** encoding works, keeping the identifiers to small values positively affects the binary size.

Gaps can be created to group fields together or keep space for future additions in the same ID range, but may negatively affect the binary size.
:::

### Field identifiers

The field identifiers combine the raw field number with an encoding marker. This one describes the following data in a very basic form, just enough to be able to skip over it, in case the field is not known to the decoder.

This encoding marker is placed in the first 3 bits and the field number in shifted to the left.

It means the maximum possible field number is **2<sup>29</sup> - 1** (**536,870,911**) instead of the integer types maximum of **2<sup>32</sup> - 1** (**4,294,967,295**). This amount is still sufficient and very unlikely to ever be reached as it is not considered realistic to have a struct or enum variant with that many fields.

The possible encodings are:

- `0`/`b000` Variable integer: Skip over individual bytes until the end marker is found.
- `1`/`b001` Length delimited: Parse the delimiter, which is always a _varint_, and skip over the length.
- Is the value a nested struct or enum?
  - Step into the nested type and skip over all its fields.
- `2`/`b010` Fixed 1-byte length: Skip over the fixed length of 1 byte (`bool`, `u8` and `i8`).
- `3`/`b011` Fixed 4-byte length: Skip over the fixed length of 4 bytes (`f32`).
- `4`/`b100` Fixed 8-byte length: Skip over the fixed length of 8 bytes (`f64`).

### Variant identifiers

The variant identifiers currently don't carry any additional information and encode the the number as is.

Therefore the current maximum possible variant number is **2<sup>32</sup> - 1** (**4,294,967,295**), although unlikely to ever be reached when using sequential numbers without gaps.
