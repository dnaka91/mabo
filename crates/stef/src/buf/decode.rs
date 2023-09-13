#![allow(clippy::type_complexity)]

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub use bytes::Buf;

use crate::varint;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum Error {
    InsufficentData,
    DecodeInt(varint::DecodeIntError),
    NonUtf8(std::string::FromUtf8Error),
    MissingField { id: u32, name: Option<&'static str> },
}

impl From<varint::DecodeIntError> for Error {
    fn from(value: varint::DecodeIntError) -> Self {
        Self::DecodeInt(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::NonUtf8(value)
    }
}

pub const END_MARKER: u32 = 0;

macro_rules! ensure_size {
    ($r:ident, $size:expr) => {
        if $r.remaining() < $size {
            return Err(Error::InsufficentData);
        }
    };
}

pub fn decode_bool(r: &mut impl Buf) -> Result<bool> {
    ensure_size!(r, 1);
    Ok(r.get_u8() != 0)
}

pub fn decode_u8(r: &mut impl Buf) -> Result<u8> {
    ensure_size!(r, 1);
    Ok(r.get_u8())
}

pub fn decode_i8(r: &mut impl Buf) -> Result<i8> {
    ensure_size!(r, 1);
    Ok(r.get_i8())
}

macro_rules! decode_int {
    ($ty:ty) => {
        paste::paste! {
            pub fn [<decode_ $ty>](r: &mut impl Buf) -> Result<$ty> {
                let (value, consumed) = varint::[<decode_ $ty>](r.chunk())?;
                r.advance(consumed);
                Ok(value)
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(decode_int!($ty);)+
    };
}

decode_int!(u16, u32, u64, u128);
decode_int!(i16, i32, i64, i128);

pub fn decode_f32(r: &mut impl Buf) -> Result<f32> {
    ensure_size!(r, 4);
    Ok(r.get_f32())
}

pub fn decode_f64(r: &mut impl Buf) -> Result<f64> {
    ensure_size!(r, 8);
    Ok(r.get_f64())
}

pub fn decode_string(r: &mut impl Buf) -> Result<String> {
    String::from_utf8(decode_bytes(r)?).map_err(Into::into)
}

pub fn decode_bytes(r: &mut impl Buf) -> Result<Vec<u8>> {
    let (len, consumed) = varint::decode_u64(r.chunk())?;
    r.advance(consumed);

    ensure_size!(r, len as usize);
    Ok(r.copy_to_bytes(len as usize).to_vec())
}

pub fn decode_vec<T: Decode>(r: &mut impl Buf) -> Result<Vec<T>> {
    let len = decode_u64(r)?;
    (0..len).map(|_| T::decode(r)).collect()
}

pub fn decode_hash_map<K: Hash + Eq + Decode, V: Decode>(
    r: &mut impl Buf,
) -> Result<HashMap<K, V>> {
    let len = decode_u64(r)?;
    (0..len)
        .map(|_| Ok((K::decode(r)?, V::decode(r)?)))
        .collect()
}

pub fn decode_hash_set<T: Hash + Eq + Decode>(r: &mut impl Buf) -> Result<HashSet<T>> {
    let len = decode_u64(r)?;
    (0..len).map(|_| T::decode(r)).collect()
}

pub fn decode_array<const N: usize, T: Debug + Decode>(r: &mut impl Buf) -> Result<[T; N]> {
    let len = decode_u64(r)?;
    if (len as usize) < N {
        return Err(Error::InsufficentData);
    }

    let buf = (0..N).map(|_| T::decode(r)).collect::<Result<Vec<_>>>()?;

    // read any remaining values, in case the old array definition was larger.
    // still have to decode the values to advance the buffer accordingly.
    for _ in N..len as usize {
        T::decode(r)?;
    }

    // SAFETY: we can unwrap here, because we ensured the Vec exactly matches
    // with the length of the array.
    Ok(buf.try_into().unwrap())
}

pub fn decode_tuple2<T0, T1>(r: &mut impl Buf) -> Result<(T0, T1)>
where
    T0: Decode,
    T1: Decode,
{
    Ok((T0::decode(r)?, T1::decode(r)?))
}

pub fn decode_tuple3<T0, T1, T2>(r: &mut impl Buf) -> Result<(T0, T1, T2)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
{
    Ok((T0::decode(r)?, T1::decode(r)?, T2::decode(r)?))
}

pub fn decode_tuple4<T0, T1, T2, T3>(r: &mut impl Buf) -> Result<(T0, T1, T2, T3)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
    ))
}

pub fn decode_tuple5<T0, T1, T2, T3, T4>(r: &mut impl Buf) -> Result<(T0, T1, T2, T3, T4)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
    ))
}

pub fn decode_tuple6<T0, T1, T2, T3, T4, T5>(r: &mut impl Buf) -> Result<(T0, T1, T2, T3, T4, T5)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
    ))
}

pub fn decode_tuple7<T0, T1, T2, T3, T4, T5, T6>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
    ))
}

pub fn decode_tuple8<T0, T1, T2, T3, T4, T5, T6, T7>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6, T7)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
        T7::decode(r)?,
    ))
}

pub fn decode_tuple9<T0, T1, T2, T3, T4, T5, T6, T7, T8>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6, T7, T8)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
        T7::decode(r)?,
        T8::decode(r)?,
    ))
}

pub fn decode_tuple10<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
        T7::decode(r)?,
        T8::decode(r)?,
        T9::decode(r)?,
    ))
}

pub fn decode_tuple11<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
    T10: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
        T7::decode(r)?,
        T8::decode(r)?,
        T9::decode(r)?,
        T10::decode(r)?,
    ))
}

pub fn decode_tuple12<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>(
    r: &mut impl Buf,
) -> Result<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)>
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
    T10: Decode,
    T11: Decode,
{
    Ok((
        T0::decode(r)?,
        T1::decode(r)?,
        T2::decode(r)?,
        T3::decode(r)?,
        T4::decode(r)?,
        T5::decode(r)?,
        T6::decode(r)?,
        T7::decode(r)?,
        T8::decode(r)?,
        T9::decode(r)?,
        T10::decode(r)?,
        T11::decode(r)?,
    ))
}

#[inline(always)]
pub fn decode_id(r: &mut impl Buf) -> Result<u32> {
    decode_u32(r)
}

pub trait Decode: Sized {
    fn decode(r: &mut impl Buf) -> Result<Self>;
}

macro_rules! forward {
    ($ty:ty) => {
        paste::paste! {
            impl Decode for $ty {
                #[inline(always)]
                fn decode(r: &mut impl Buf) -> Result<Self> {
                    [<decode_ $ty>](r)
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

impl Decode for Box<str> {
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_string(r).map(String::into_boxed_str)
    }
}

impl Decode for Box<[u8]> {
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_bytes(r).map(Vec::into_boxed_slice)
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_vec(r)
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Hash + Eq + Decode,
    V: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_hash_map(r)
    }
}

impl<T> Decode for HashSet<T>
where
    T: Hash + Eq + Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_hash_set(r)
    }
}

impl<const N: usize, T> Decode for [T; N]
where
    T: Debug + Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_array(r)
    }
}

impl<T0, T1> Decode for (T0, T1)
where
    T0: Decode,
    T1: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple2(r)
    }
}

impl<T0, T1, T2> Decode for (T0, T1, T2)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple3(r)
    }
}

impl<T0, T1, T2, T3> Decode for (T0, T1, T2, T3)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple4(r)
    }
}

impl<T0, T1, T2, T3, T4> Decode for (T0, T1, T2, T3, T4)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple5(r)
    }
}

impl<T0, T1, T2, T3, T4, T5> Decode for (T0, T1, T2, T3, T4, T5)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple6(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> Decode for (T0, T1, T2, T3, T4, T5, T6)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple7(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> Decode for (T0, T1, T2, T3, T4, T5, T6, T7)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple8(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8> Decode for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple9(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> Decode for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple10(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Decode
    for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
    T10: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple11(r)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> Decode
    for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
where
    T0: Decode,
    T1: Decode,
    T2: Decode,
    T3: Decode,
    T4: Decode,
    T5: Decode,
    T6: Decode,
    T7: Decode,
    T8: Decode,
    T9: Decode,
    T10: Decode,
    T11: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_tuple12(r)
    }
}

// -----------------------------
// TODO: NonZero
// -----------------------------

impl<'a, T> Decode for std::borrow::Cow<'a, T>
where
    T: Copy + Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        T::decode(r).map(std::borrow::Cow::Owned)
    }
}

impl<T> Decode for std::rc::Rc<T>
where
    T: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        T::decode(r).map(std::rc::Rc::new)
    }
}

impl<T> Decode for std::sync::Arc<T>
where
    T: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        T::decode(r).map(std::sync::Arc::new)
    }
}
