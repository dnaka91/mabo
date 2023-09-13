use std::{
    collections::{HashMap, HashSet},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
};

pub use bytes::BufMut;

use crate::varint;

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

pub fn encode_vec<T: Encode>(w: &mut impl BufMut, vec: &[T]) {
    encode_u64(w, vec.len() as u64);

    for value in vec {
        value.encode(w);
    }
}

pub fn encode_hash_map<K: Encode, V: Encode>(w: &mut impl BufMut, map: &HashMap<K, V>) {
    encode_u64(w, map.len() as u64);

    for (key, value) in map {
        key.encode(w);
        value.encode(w);
    }
}

pub fn encode_hash_set<T: Encode>(w: &mut impl BufMut, set: &HashSet<T>) {
    encode_u64(w, set.len() as u64);

    for value in set {
        value.encode(w);
    }
}

pub fn encode_option<T: Encode>(w: &mut impl BufMut, option: &Option<T>) {
    if let Some(value) = option {
        value.encode(w);
    }
}

pub fn encode_array<const N: usize, T: Encode>(w: &mut impl BufMut, array: &[T; N]) {
    encode_u64(w, array.len() as u64);

    for value in array {
        value.encode(w);
    }
}

pub fn encode_tuple2<T0, T1>(w: &mut impl BufMut, tuple: &(T0, T1))
where
    T0: Encode,
    T1: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
}

pub fn encode_tuple3<T0, T1, T2>(w: &mut impl BufMut, tuple: &(T0, T1, T2))
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
}

pub fn encode_tuple4<T0, T1, T2, T3>(w: &mut impl BufMut, tuple: &(T0, T1, T2, T3))
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
}

pub fn encode_tuple5<T0, T1, T2, T3, T4>(w: &mut impl BufMut, tuple: &(T0, T1, T2, T3, T4))
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
}

pub fn encode_tuple6<T0, T1, T2, T3, T4, T5>(w: &mut impl BufMut, tuple: &(T0, T1, T2, T3, T4, T5))
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
}

pub fn encode_tuple7<T0, T1, T2, T3, T4, T5, T6>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
}

pub fn encode_tuple8<T0, T1, T2, T3, T4, T5, T6, T7>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6, T7),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
    tuple.7.encode(w);
}

pub fn encode_tuple9<T0, T1, T2, T3, T4, T5, T6, T7, T8>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6, T7, T8),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
    tuple.7.encode(w);
    tuple.8.encode(w);
}

pub fn encode_tuple10<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
    tuple.7.encode(w);
    tuple.8.encode(w);
    tuple.9.encode(w);
}

pub fn encode_tuple11<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
    T10: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
    tuple.7.encode(w);
    tuple.8.encode(w);
    tuple.9.encode(w);
    tuple.10.encode(w);
}

#[allow(clippy::type_complexity)]
pub fn encode_tuple12<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>(
    w: &mut impl BufMut,
    tuple: &(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
) where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
    T10: Encode,
    T11: Encode,
{
    tuple.0.encode(w);
    tuple.1.encode(w);
    tuple.2.encode(w);
    tuple.3.encode(w);
    tuple.4.encode(w);
    tuple.5.encode(w);
    tuple.6.encode(w);
    tuple.7.encode(w);
    tuple.8.encode(w);
    tuple.9.encode(w);
    tuple.10.encode(w);
    tuple.11.encode(w);
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

pub trait Encode {
    fn encode(&self, w: &mut impl BufMut);
}

macro_rules! forward {
    ($ty:ty) => {
        paste::paste! {
            impl Encode for $ty {
                #[inline(always)]
                fn encode(&self, w: &mut impl BufMut) {
                    [<encode_ $ty>](w, *self)
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

impl Encode for Box<str> {
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_string(w, self)
    }
}

impl Encode for Box<[u8]> {
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_bytes(w, self)
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self)
    }
}

impl<'a, T> Encode for &'a [T]
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_vec(w, self)
    }
}

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_map(w, self)
    }
}

impl<T> Encode for HashSet<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_hash_set(w, self)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_option(w, self)
    }
}

impl<const N: usize, T> Encode for [T; N]
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_array(w, self)
    }
}

impl<T0, T1> Encode for (T0, T1)
where
    T0: Encode,
    T1: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple2(w, self)
    }
}

impl<T0, T1, T2> Encode for (T0, T1, T2)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple3(w, self)
    }
}

impl<T0, T1, T2, T3> Encode for (T0, T1, T2, T3)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple4(w, self)
    }
}

impl<T0, T1, T2, T3, T4> Encode for (T0, T1, T2, T3, T4)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple5(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5> Encode for (T0, T1, T2, T3, T4, T5)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple6(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> Encode for (T0, T1, T2, T3, T4, T5, T6)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple7(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> Encode for (T0, T1, T2, T3, T4, T5, T6, T7)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple8(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8> Encode for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple9(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> Encode for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple10(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Encode
    for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
    T10: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple11(w, self)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> Encode
    for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
where
    T0: Encode,
    T1: Encode,
    T2: Encode,
    T3: Encode,
    T4: Encode,
    T5: Encode,
    T6: Encode,
    T7: Encode,
    T8: Encode,
    T9: Encode,
    T10: Encode,
    T11: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        encode_tuple12(w, self)
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

impl<'a, T> Encode for std::borrow::Cow<'a, T>
where
    T: Clone + Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w)
    }
}

impl<T> Encode for std::rc::Rc<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w)
    }
}

impl<T> Encode for std::sync::Arc<T>
where
    T: Encode,
{
    #[inline(always)]
    fn encode(&self, w: &mut impl BufMut) {
        T::encode(self, w)
    }
}
