#![allow(
    missing_docs,
    clippy::cast_possible_truncation,
    clippy::implicit_hasher,
    clippy::inline_always,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

pub use buf::{Buf, BufMut, Bytes, Decode, Encode};

pub mod buf;
pub mod varint;

#[derive(Clone, Copy)]
pub struct FieldId {
    pub value: u32,
    pub encoding: FieldEncoding,
}

impl FieldId {
    #[inline]
    #[must_use]
    pub const fn new(value: u32, encoding: FieldEncoding) -> Self {
        Self { value, encoding }
    }

    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        let Some(encoding) = FieldEncoding::from_u32(value) else {
            return None;
        };
        let value = value >> 3;

        Some(Self { value, encoding })
    }

    #[inline]
    #[must_use]
    pub const fn into_u32(self) -> u32 {
        self.value << 3 | self.encoding as u32
    }
}

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

#[derive(Clone, Copy)]
pub struct VariantId {
    pub value: u32,
}

impl VariantId {
    #[inline]
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[macro_export]
macro_rules! include {
    ($name:literal) => {
        include!(concat!(env!("OUT_DIR"), "/stef/", $name, ".rs"));
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct NonZero<T>(T);

impl<T> NonZero<T> {
    /// ```
    /// let value = stef::NonZeroString::new("hello".to_owned()).unwrap();
    /// assert_eq!("hello", value.get());
    /// ```
    pub fn get(&self) -> &T {
        &self.0
    }

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

pub type NonZeroString = NonZero<String>;
pub type NonZeroBytes = NonZero<Vec<u8>>;
pub type NonZeroVec<T> = NonZero<Vec<T>>;
pub type NonZeroHashMap<K, V> = NonZero<HashMap<K, V>>;
pub type NonZeroHashSet<T> = NonZero<HashSet<T>>;
