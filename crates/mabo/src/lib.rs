//! Runtime support crate for the Mabo encoding format.

#![allow(
    clippy::cast_possible_truncation,
    clippy::implicit_hasher,
    clippy::inline_always,
    clippy::module_name_repetitions
)]

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

pub use buf::{Buf, BufMut, Bytes, Decode, Encode};

pub mod buf;
pub mod varint;

/// Identifier for a single struct or enum variant field.
///
/// This type contains the actual identifier, plus additional information that is encoded together
/// with it. It allows for convenient en- and decoding of the information.
#[derive(Clone, Copy)]
pub struct FieldId {
    /// The real decoded field identifier.
    pub value: u32,
    /// Encoding information for field skipping.
    pub encoding: FieldEncoding,
}

impl FieldId {
    /// Create a new instance of a field identifier.
    #[inline]
    #[must_use]
    pub const fn new(value: u32, encoding: FieldEncoding) -> Self {
        Self { value, encoding }
    }

    /// Convert from a raw `u32` into the field identifier.
    ///
    /// This returns `None` if the raw value contains an unknown field encoding.
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        let Some(encoding) = FieldEncoding::from_u32(value) else {
            return None;
        };
        let value = value >> 3;

        Some(Self { value, encoding })
    }

    /// Convert the field identifier into a raw `u32`, which contains all its information.
    #[inline]
    #[must_use]
    pub const fn into_u32(self) -> u32 {
        self.value << 3 | self.encoding as u32
    }
}

/// Minimum detail about how a field is encoded, which allows to skip over a field if it's unknown.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum FieldEncoding {
    /// Variable-length integer.
    Varint = 0,
    /// Arbitrary content prefixed with its byte length.
    LengthPrefixed = 1,
    /// 1-byte fixed width data.
    Fixed1 = 2,
    /// 4-byte fixed width data.
    Fixed4 = 3,
    /// 8-byte fixed width data.
    Fixed8 = 4,
}

impl FieldEncoding {
    /// Try to convert the raw field identifier (which carries the encoding information) into a
    /// known field encoding.
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        Some(match value & 0b111 {
            0 => Self::Varint,
            1 => Self::LengthPrefixed,
            2 => Self::Fixed1,
            3 => Self::Fixed4,
            4 => Self::Fixed8,
            _ => return None,
        })
    }
}

/// Identifier for a single enum variant.
///
/// Currently, this is identical to the raw value it contains, but might be extended to encode
/// additional information like the [`FieldId`] in the future.
#[derive(Clone, Copy)]
pub struct VariantId {
    /// The real decoded variant identifier.
    pub value: u32,
}

impl VariantId {
    /// Create a new instance of a variant identifier.
    #[inline]
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Convenience macro to include generated Rust code for Mabo schemas.
///
/// By default build scripts write output files into a special directory provided by Cargo as
/// `OUT_DIR` environment variable. The `mabo-build` crate additional puts all generated files into
/// a `mabo/` sub-folder instead of placing the files at the root.
///
/// The final file name is derived from the schema file name as it is the same, except for the
/// different file extension.
///
/// # Example
///
/// Assuming a schema file named `sample.mabo` that is being compiled the `mabo-build` crate, the
/// final output destination would be:
///
/// ```txt
/// $OUT_DIR/mabo/sample.rs
/// ```
#[macro_export]
macro_rules! include {
    ($name:literal) => {
        include!(concat!(env!("OUT_DIR"), "/mabo/", $name, ".rs"));
    };
}

/// A container that guarantees that the contained type is not zero or not empty.
///
/// What's exactly meant by _non-zero_ depends on the type itself. For example, integers define this
/// as value that are not literally `0` (but those are handled by Rust's built-in `NonZeroN` types
/// anyway), and collections define this as not being empty, meaning to always contain at least one
/// element. Similarly for strings, it means they always contain at least one character.
#[derive(Clone, Debug, PartialEq)]
pub struct NonZero<T>(T);

impl<T> NonZero<T> {
    /// ```
    /// let value = mabo::NonZero::<String>::new("hello".to_owned()).unwrap();
    /// assert_eq!("hello", value.get());
    /// ```
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Extract the inner type out of the non-zero container, but lose the guarantee of not being
    /// zero.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for NonZero<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! non_zero_collection {
    ($name:ident $(< $($gens:tt),+ >)?) => {
        impl $(< $($gens),+ >)? NonZero<$name $(< $($gens),+ >)?> {
            /// Try to create a new non-zero instance, which will succeed if the given collection
            /// contains in fact some elements. Otherwise `None` is returned.
            #[must_use]
            pub fn new(value: $name $(< $($gens),+ >)?) -> Option<Self> {
                (!value.is_empty()).then_some(Self(value))
            }
        }
    };
}

non_zero_collection!(String);
non_zero_collection!(Vec<T>);
non_zero_collection!(Bytes);
non_zero_collection!(HashMap<K, V>);
non_zero_collection!(HashSet<T>);

/// String (Mabo's `non_zero<string>`) that is guaranteed to not be empty.
pub type NonZeroString = NonZero<String>;
/// Byte vector (Mabo's `non_zero<bytes>`) that is guaranteed to not be empty.
pub type NonZeroBytes = NonZero<Vec<u8>>;
/// Vector of `T` (Mabo's `non_zero<vec<T>>`) that is guaranteed to not be empty.
pub type NonZeroVec<T> = NonZero<Vec<T>>;
/// Hash map (Mabo's `non_zero<hash_map<K ,V>>`) that is guaranteed to not be empty.
pub type NonZeroHashMap<K, V> = NonZero<HashMap<K, V>>;
/// Hash set (Mabo's `non_zero<hash_set<T>>`) that is guaranteed to not be empty.
pub type NonZeroHashSet<T> = NonZero<HashSet<T>>;
