use std::{
    collections::{HashMap, HashSet},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
};

pub use bytes::BufMut;

use crate::{varint, NonZero};

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
    encode_bytes(w, value.as_bytes());
}

pub fn encode_bytes(w: &mut impl BufMut, value: &[u8]) {
    encode_u64(w, value.len() as u64);
    w.put(value)
}

pub fn encode_vec<W, T, E>(w: &mut W, vec: &[T], encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    encode_u64(w, vec.len() as u64);

    for value in vec {
        encode(w, value);
    }
}

pub fn encode_hash_map<W, K, V, EK, EV>(
    w: &mut W,
    map: &HashMap<K, V>,
    encode_key: EK,
    encode_value: EV,
) where
    W: BufMut,
    EK: Fn(&mut W, &K),
    EV: Fn(&mut W, &V),
{
    encode_u64(w, map.len() as u64);

    for (key, value) in map {
        encode_key(w, key);
        encode_value(w, value);
    }
}

pub fn encode_hash_set<W, T, E>(w: &mut W, set: &HashSet<T>, encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    encode_u64(w, set.len() as u64);

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

pub fn encode_array<const N: usize, W, T, E>(w: &mut W, array: &[T; N], encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    encode_u64(w, array.len() as u64);

    for value in array {
        encode(w, value);
    }
}

#[inline(always)]
pub fn encode_id(w: &mut impl BufMut, id: u32) {
    encode_u32(w, id)
}

#[inline(always)]
pub fn encode_field<W, E>(w: &mut W, id: u32, encode: E)
where
    W: BufMut,
    E: Fn(&mut W),
{
    encode_id(w, id);
    encode(w);
}

#[inline(always)]
pub fn encode_field_option<W, T, E>(w: &mut W, id: u32, option: &Option<T>, encode: E)
where
    W: BufMut,
    E: Fn(&mut W, &T),
{
    if let Some(value) = option {
        encode_id(w, id);
        encode(w, value);
    }
}

pub trait Encode {
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
        encode_bytes(w, self);
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self, |w, v| v.encode(w));
    }
}

impl<'a, T> Encode for &'a [T]
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self, |w, v| v.encode(w));
    }
}

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_map(w, self, |w, k| k.encode(w), |w, v| v.encode(w));
    }
}

impl<T> Encode for HashSet<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_set(w, self, |w, v| v.encode(w));
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
        encode_array(w, self, |w, v| v.encode(w));
    }
}

impl Encode for NonZeroU8 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_u8(w, self.get());
    }
}

impl Encode for NonZeroU16 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_u16(w, self.get());
    }
}

impl Encode for NonZeroU32 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_u32(w, self.get());
    }
}

impl Encode for NonZeroU64 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_u64(w, self.get());
    }
}

impl Encode for NonZeroU128 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_u128(w, self.get());
    }
}

impl Encode for NonZeroI8 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_i8(w, self.get());
    }
}

impl Encode for NonZeroI16 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_i16(w, self.get());
    }
}

impl Encode for NonZeroI32 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_i32(w, self.get());
    }
}

impl Encode for NonZeroI64 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_i64(w, self.get());
    }
}

impl Encode for NonZeroI128 {
    fn encode(&self, w: &mut impl BufMut) {
        encode_i128(w, self.get());
    }
}

impl<T> Encode for NonZero<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        self.0.encode(w)
    }
}

impl<'a, T> Encode for std::borrow::Cow<'a, T>
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
