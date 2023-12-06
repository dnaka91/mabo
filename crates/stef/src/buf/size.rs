use std::collections::{HashMap, HashSet};

use bytes::Bytes;

use crate::{varint, NonZero};

macro_rules! size_fixed {
    ($ty:ty => $size:literal) => {
        paste::paste! {
            #[inline(always)]
            #[must_use]
            pub const fn [<size_ $ty>](_: $ty) -> usize {
                $size
            }
        }
    };
    ($($ty:ty => $size:literal),+ $(,)?) => {
        $(size_fixed!($ty => $size);)+
    };
}

size_fixed!(
    bool => 1,
    u8 => 1,
    i8 => 1,
    f32 => 4,
    f64 => 8,
);

macro_rules! size_int {
    ($ty:ty) => {
        paste::paste! {
            #[must_use]
            pub const fn [<size_ $ty>](value: $ty) -> usize {
                varint::[<size_ $ty>](value)
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(size_int!($ty);)+
    };
}

size_int!(u16, u32, u64, u128);
size_int!(i16, i32, i64, i128);

#[must_use]
pub const fn size_string(value: &str) -> usize {
    size_bytes_std(value.as_bytes())
}

#[must_use]
pub const fn size_bytes_std(value: &[u8]) -> usize {
    size_u64(value.len() as u64) + value.len()
}

pub const fn size_bytes_bytes(value: &Bytes) -> usize {
    size_u64(value.len() as u64) + value.len()
}

pub fn size_vec<T, S>(vec: &[T], size: S) -> usize
where
    S: Fn(&T) -> usize,
{
    size_u64(vec.len() as u64) + vec.iter().map(size).sum::<usize>()
}

pub fn size_hash_map<K, V, SK, SV>(map: &HashMap<K, V>, size_key: SK, size_value: SV) -> usize
where
    SK: Fn(&K) -> usize,
    SV: Fn(&V) -> usize,
{
    size_u64(map.len() as u64)
        + map
            .iter()
            .map(|(key, value)| size_key(key) + size_value(value))
            .sum::<usize>()
}

pub fn size_hash_set<T, S>(set: &HashSet<T>, size: S) -> usize
where
    S: Fn(&T) -> usize,
{
    size_u64(set.len() as u64) + set.iter().map(size).sum::<usize>()
}

pub fn size_option<T, S>(option: Option<&T>, size: S) -> usize
where
    S: Fn(&T) -> usize,
{
    size_u8(0) + option.map_or(0, size)
}

pub fn size_array<const N: usize, T, S>(array: &[T; N], size: S) -> usize
where
    S: Fn(&T) -> usize,
{
    size_u64(N as u64) + array.iter().map(size).sum::<usize>()
}

#[inline(always)]
#[must_use]
pub fn size_id(id: u32) -> usize {
    size_u32(id)
}

#[inline(always)]
pub fn size_field<S>(id: u32, size: S) -> usize
where
    S: Fn() -> usize,
{
    size_id(id) + size()
}

#[inline(always)]
pub fn size_field_option<T, S>(id: u32, option: Option<&T>, size: S) -> usize
where
    S: Fn(&T) -> usize,
{
    option.map_or(0, |value| size_id(id) + size(value))
}

pub trait Size {
    fn size(&self) -> usize;
}

macro_rules! forward {
    ($ty:ty) => {
        paste::paste! {
            impl Size for $ty {
                #[inline(always)]
                fn size(&self) -> usize {
                    [<size_ $ty>](*self)
                }
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(forward!($ty);)+
    };
}

forward!(bool);
forward!(u8, u16, u32, u64, u128);
forward!(i8, i16, i32, i64, i128);
forward!(f32, f64);

impl Size for String {
    #[inline(always)]
    fn size(&self) -> usize {
        size_string(self)
    }
}

impl Size for Box<str> {
    #[inline(always)]
    fn size(&self) -> usize {
        size_string(self)
    }
}

impl Size for Box<[u8]> {
    #[inline(always)]
    fn size(&self) -> usize {
        size_bytes_std(self)
    }
}

impl<T> Size for Vec<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_vec(self, Size::size)
    }
}

impl<T> Size for &'_ [T]
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_vec(self, Size::size)
    }
}

impl<K, V> Size for HashMap<K, V>
where
    K: Size,
    V: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_hash_map(self, Size::size, Size::size)
    }
}

impl<T> Size for HashSet<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_hash_set(self, Size::size)
    }
}

impl<T> Size for Option<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_option(self.as_ref(), Size::size)
    }
}

impl<const N: usize, T> Size for [T; N]
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        size_array(self, Size::size)
    }
}

impl<T> Size for NonZero<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        self.0.size()
    }
}

impl<T> Size for std::borrow::Cow<'_, T>
where
    T: Clone + Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        T::size(self)
    }
}

impl<T> Size for std::rc::Rc<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        T::size(self)
    }
}

impl<T> Size for std::sync::Arc<T>
where
    T: Size,
{
    #[inline(always)]
    fn size(&self) -> usize {
        T::size(self)
    }
}
