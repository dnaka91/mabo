use std::collections::{HashMap, HashSet};

pub use bytes::{BufMut, Bytes};

use crate::{varint, FieldId, NonZero, VariantId};

pub fn encode_bool(w: &mut impl BufMut, value: bool) {
    w.put_u8(value.into());
}

pub fn encode_u8(w: &mut impl BufMut, value: u8) {
    w.put_u8(value);
}

pub fn encode_i8(w: &mut impl BufMut, value: i8) {
    w.put_i8(value);
}

macro_rules! encode_int {
    ($ty:ty) => {
        paste::paste! {
            pub fn [<encode_ $ty>](w: &mut impl BufMut, value: $ty) {
                let (buf, len) = varint::[<encode_ $ty>](value);
                w.put(&buf[..len]);
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(encode_int!($ty);)+
    };
}

encode_int!(u16, u32, u64, u128);
encode_int!(i16, i32, i64, i128);

pub fn encode_f32(w: &mut impl BufMut, value: f32) {
    w.put_f32(value);
}

pub fn encode_f64(w: &mut impl BufMut, value: f64) {
    w.put_f64(value);
}

pub fn encode_string(w: &mut impl BufMut, value: &str) {
    encode_bytes_std(w, value.as_bytes());
}

pub fn encode_bytes_std(w: &mut impl BufMut, value: &[u8]) {
    encode_u64(w, value.len() as u64);
    w.put(value);
}

pub fn encode_bytes_bytes(w: &mut impl BufMut, value: &Bytes) {
    encode_bytes_std(w, value);
}

pub fn encode_vec<W, T, S, E>(w: &mut W, vec: &[T], size: S, encode: E)
where
    W: BufMut,
    S: Fn(&T) -> usize,
    E: Fn(&mut W, &T),
{
    encode_u64(w, vec.iter().map(size).sum::<usize>() as u64);

    for value in vec {
        encode(w, value);
    }
}

pub fn encode_hash_map<W, K, V, SK, SV, EK, EV>(
    w: &mut W,
    map: &HashMap<K, V>,
    size_key: SK,
    size_value: SV,
    encode_key: EK,
    encode_value: EV,
) where
    W: BufMut,
    SK: Fn(&K) -> usize,
    SV: Fn(&V) -> usize,
    EK: Fn(&mut W, &K),
    EV: Fn(&mut W, &V),
{
    encode_u64(
        w,
        map.iter()
            .map(|(k, v)| size_key(k) + size_value(v))
            .sum::<usize>() as u64,
    );

    for (key, value) in map {
        encode_key(w, key);
        encode_value(w, value);
    }
}

pub fn encode_hash_set<W, T, S, E>(w: &mut W, set: &HashSet<T>, size: S, encode: E)
where
    W: BufMut,
    S: Fn(&T) -> usize,
    E: Fn(&mut W, &T),
{
    encode_u64(w, set.iter().map(size).sum::<usize>() as u64);

    for value in set {
        encode(w, value);
    }
}

pub fn encode_option<W, T, E>(w: &mut W, option: &Option<T>, encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    if let Some(value) = option {
        w.put_u8(1);
        encode(w, value);
    } else {
        w.put_u8(0);
    }
}

pub fn encode_array<const N: usize, W, T, S, E>(w: &mut W, array: &[T; N], size: S, encode: E)
where
    W: BufMut,
    S: Fn(&T) -> usize,
    E: Fn(&mut W, &T),
{
    encode_u64(w, array.iter().map(size).sum::<usize>() as u64);

    for value in array {
        encode(w, value);
    }
}

#[inline(always)]
pub fn encode_tuple<W, S, E>(w: &mut W, size: S, encode: E)
where
    W: BufMut,
    S: Fn() -> usize,
    E: Fn(&mut W),
{
    encode_u64(w, size() as u64);
    encode(w);
}

#[inline(always)]
pub fn encode_id(w: &mut impl BufMut, id: FieldId) {
    encode_u32(w, id.into_u32());
}

#[inline(always)]
pub fn encode_variant_id(w: &mut impl BufMut, id: VariantId) {
    encode_u32(w, id.value);
}

#[inline(always)]
pub fn encode_field<W, E>(w: &mut W, id: FieldId, encode: E)
where
    W: BufMut,
    E: Fn(&mut W),
{
    encode_id(w, id);
    encode(w);
}

#[inline(always)]
pub fn encode_field_option<W, T, E>(w: &mut W, id: FieldId, option: &Option<T>, encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    if let Some(value) = option {
        encode_id(w, id);
        encode(w, value);
    }
}

pub trait Encode: super::Size {
    fn encode(&self, w: &mut impl BufMut);
}

macro_rules! forward {
    ($ty:ty) => {
        paste::paste! {
            impl Encode for $ty {
                #[inline(always)]
                fn encode(&self, w: &mut impl BufMut) {
                    [<encode_ $ty>](w, *self);
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

impl Encode for String {
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_string(w, self);
    }
}

impl Encode for Box<str> {
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_string(w, self);
    }
}

impl Encode for Box<[u8]> {
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_bytes_std(w, self);
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self, T::size, |w, v| v.encode(w));
    }
}

impl<T> Encode for &'_ [T]
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self, T::size, |w, v| v.encode(w));
    }
}

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_map(
            w,
            self,
            K::size,
            V::size,
            |w, k| k.encode(w),
            |w, v| v.encode(w),
        );
    }
}

impl<T> Encode for HashSet<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_set(w, self, T::size, |w, v| v.encode(w));
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_option(w, self, |w, v| v.encode(w));
    }
}

impl<const N: usize, T> Encode for [T; N]
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_array(w, self, T::size, |w, v| v.encode(w));
    }
}

impl<T> Encode for NonZero<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        self.0.encode(w);
    }
}

impl<T> Encode for std::borrow::Cow<'_, T>
where
    T: Clone + Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w);
    }
}

impl<T> Encode for std::rc::Rc<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w);
    }
}

impl<T> Encode for std::sync::Arc<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w);
    }
}
