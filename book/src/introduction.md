---
order: 1
---

# Introduction

Strongly typed and schema-based binary data format, with a type system as strong as Rust's.

Mabo is a data encoding format, that borrows a lot from existing schema-based formats like [Protobuf](https://protobuf.dev), [Cap'n Proto](https://capnproto.org) and [Flatbuffers](https://flatbuffers.dev). It takes some ideas from the binary data formats [Postcard](https://github.com/jamesmunns/postcard) and [Bincode](https://github.com/bincode-org/bincode) as well, which are popular choices in the Rust ecosystem.

In contrast to those projects it favors a very strong and strict type system, and takes different overall design decisions. It is not necessarily _better_ than them, but takes other approaches to the challenges when designing a data format.

## What means schema-based?

Data formats can generally be categorized into 3 groups: Self-describing, non-self-describing, and schema-based.

Self-describing formats are most common in text-based formats, meaning they're human readable. The self-describing part is usually implemented by either pairing each value with a key to name it, or writing a descriptor for the format at the beginning of a payload. Popular formats include _JSON_, _XML_, _CSV_ and many more.

Non-self-describing formats are most common in binary formats that are machine readable. They are often accompany special handling of the payload to squeeze out extra saving in storage size and the structure is often defined by the program itself. Some popular formats with a defined structure are _MessagePack_ or _CBOR_, but again, there are many more.

Lastly schema-based formats are mostly binary as well, but their structure is described in a text-based schema. This schema is then used to generate the source code that defines data structures and the logic to encode and decode it into and from the format.

Within all these groups there are exceptions as well. There are text-based formats that are not self-describing and there are binary formats that are self-describing or offer to encode as either self-describing or not.

Mabo falls into the last group.

## Why a stronger type system?

Firs and foremost, I personally really enjoy the Rust programming language and its strict but flexible type system.

In the many years that I have used Protobufs, they always have felt limited with the few supported built-in types. Most of the time that resulted in an additional round of validation after decoding the format, to ensure all data is in a valid form.

Many of these validations could be avoided by a stronger type system, which would rule out many wrong states by refusing them upfront.

For example consider a the type for a TCP port. It can range from `0` to `65535` (inclusive), making a 16-bit unsigned integer the perfect fit for the value range. Many formats don't provide a data type for this integer, despite many programming language having one (`u16`, `uint16_t`, `uint16`, `Short`, ...).

By extending the type system, data structures can be defined in a way, that ensures the data is already in a valid state after decoding, not needing an additional step for validation.

## The project name

The name Mabo is Japanese (_マーボー_) and a short form of the chinese dish [麻婆豆腐](https://en.wikipedia.org/wiki/Mapo_tofu) ([_マーボーどうふ_](https://jisho.org/word/%E9%BA%BB%E5%A9%86%E8%B1%86%E8%85%90), [_/maːboːdoɯɸɯ/_](https://en.wikipedia.org/wiki/Help:IPA/Japanese), _mabodofu_), which was one of my favorite foods while living in Japan.

After long search for a project name I gave up with acronyms and decided to follow naming after food, as done with [Bun](https://bun.sh) and [OpenTofu](https://opentofu.org). The Bun logo resembles a [肉まん](https://en.wikipedia.org/wiki/Baozi) ([_にくまん_](https://jisho.org/word/%E8%82%89%E9%A5%85), [_/nikɯmaɴ/_](https://en.wikipedia.org/wiki/Help:IPA/Japanese), _nikuman_) with is face on it, which is a soft bread filled with pork, and the OpenTofu logo resembles a block of tofu with indications of eyes.

Therefore, I decided to name the project after one of my favorite dishes.
