#![allow(clippy::type_complexity)]

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub use bytes::{Buf, Bytes};

use crate::{varint, FieldEncoding, FieldId, NonZero, VariantId};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    InsufficientData,
    DecodeInt(varint::DecodeIntError),
    NonUtf8(std::string::FromUtf8Error),
    MissingField { id: u32, name: Option<&'static str> },
    UnknownVariant(u32),
    UnknownEncoding(u32),
    Zero,
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
            return Err(Error::InsufficientData);
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
    String::from_utf8(decode_bytes_std(r)?).map_err(Into::into)
}

pub fn decode_bytes_std(r: &mut impl Buf) -> Result<Vec<u8>> {
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    Ok(r.copy_to_bytes(len as usize).to_vec())
}

pub fn decode_bytes_bytes(r: &mut impl Buf) -> Result<Bytes> {
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    Ok(r.copy_to_bytes(len as usize))
}

pub fn decode_vec<R, T, D>(r: &mut R, decode: D) -> Result<Vec<T>>
where
    R: Buf,
    D: Fn(&mut bytes::buf::Take<&mut R>) -> Result<T>,
{
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    let mut vec = Vec::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        vec.push(decode(&mut r)?);
    }

    Ok(vec)
}

pub fn decode_hash_map<R, K, V, DK, DV>(
    r: &mut R,
    decode_key: DK,
    decode_value: DV,
) -> Result<HashMap<K, V>>
where
    R: Buf,
    K: Hash + Eq,
    DK: Fn(&mut bytes::buf::Take<&mut R>) -> Result<K>,
    DV: Fn(&mut bytes::buf::Take<&mut R>) -> Result<V>,
{
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    let mut map = HashMap::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        map.insert(decode_key(&mut r)?, decode_value(&mut r)?);
    }

    Ok(map)
}

pub fn decode_hash_set<R, T, D>(r: &mut R, decode: D) -> Result<HashSet<T>>
where
    R: Buf,
    T: Hash + Eq,
    D: Fn(&mut bytes::buf::Take<&mut R>) -> Result<T>,
{
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    let mut set = HashSet::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        set.insert(decode(&mut r)?);
    }

    Ok(set)
}

pub fn decode_option<R, T, D>(r: &mut R, decode: D) -> Result<Option<T>>
where
    R: Buf,
    D: Fn(&mut R) -> Result<T>,
{
    let some = decode_u8(r)? == 1;
    if some {
        decode(r).map(Some)
    } else {
        Ok(None)
    }
}

pub fn decode_array<const N: usize, R, T, D>(r: &mut R, decode: D) -> Result<[T; N]>
where
    R: Buf,
    T: Debug,
    D: Fn(&mut bytes::buf::Take<&mut R>) -> Result<T>,
{
    let len = decode_u64(r)?;
    ensure_size!(r, len as usize);

    let mut vec = Vec::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() && vec.len() < N {
        vec.push(decode(&mut r)?);
    }

    // skip any remaining values, in case the old array definition was larger.
    r.advance(r.remaining());

    // SAFETY: we can unwrap here, because we ensured the Vec exactly matches
    // with the length of the array.
    Ok(vec.try_into().unwrap())
}

macro_rules! ensure_not_empty {
    ($size:ident) => {
        if $size == 0 {
            return Err(Error::Zero);
        }
    };
}

macro_rules! decode_non_zero_int {
    ($ty:ty) => {
        paste::paste! {
            pub fn [<decode_non_zero_ $ty>](
                r: &mut impl Buf,
            ) -> Result<std::num::[<NonZero $ty:upper>]> {
                std::num::[<NonZero $ty:upper>]::new([<decode_ $ty>](r)?)
                    .ok_or_else(|| Error::Zero)
            }
        }
    };
    ($($ty:ty),+ $(,)?) => {
        $(decode_non_zero_int!($ty);)+
    };
}

decode_non_zero_int!(u8, u16, u32, u64, u128);
decode_non_zero_int!(i8, i16, i32, i64, i128);

pub fn decode_non_zero_string(r: &mut impl Buf) -> Result<NonZero<String>> {
    String::from_utf8(decode_non_zero_bytes_std(r)?.into_inner())
        .map(|v| NonZero::<String>::new(v).unwrap())
        .map_err(Into::into)
}

pub fn decode_non_zero_bytes_std(r: &mut impl Buf) -> Result<NonZero<Vec<u8>>> {
    let len = decode_u64(r)?;
    ensure_not_empty!(len);
    ensure_size!(r, len as usize);

    Ok(NonZero::<Vec<_>>::new(r.copy_to_bytes(len as usize).to_vec()).unwrap())
}

pub fn decode_non_zero_bytes_bytes(r: &mut impl Buf) -> Result<NonZero<Bytes>> {
    let len = decode_u64(r)?;
    ensure_not_empty!(len);
    ensure_size!(r, len as usize);

    Ok(NonZero::<Bytes>::new(r.copy_to_bytes(len as usize)).unwrap())
}

pub fn decode_non_zero_vec<R, T, D>(r: &mut R, decode: D) -> Result<NonZero<Vec<T>>>
where
    R: Buf,
    D: Fn(&mut bytes::buf::Take<&mut R>) -> Result<T>,
{
    let len = decode_u64(r)?;
    ensure_not_empty!(len);
    ensure_size!(r, len as usize);

    let mut vec = Vec::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        vec.push(decode(&mut r)?);
    }

    Ok(NonZero::<Vec<_>>::new(vec).unwrap())
}

pub fn decode_non_zero_hash_map<R, K, V, DK, DV>(
    r: &mut R,
    decode_key: DK,
    decode_value: DV,
) -> Result<NonZero<HashMap<K, V>>>
where
    R: Buf,
    K: Hash + Eq,
    DK: Fn(&mut bytes::buf::Take<&mut R>) -> Result<K>,
    DV: Fn(&mut bytes::buf::Take<&mut R>) -> Result<V>,
{
    let len = decode_u64(r)?;
    ensure_not_empty!(len);
    ensure_size!(r, len as usize);

    let mut map = HashMap::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        map.insert(decode_key(&mut r)?, decode_value(&mut r)?);
    }

    Ok(NonZero::<HashMap<_, _>>::new(map).unwrap())
}

pub fn decode_non_zero_hash_set<R, T, D>(r: &mut R, decode: D) -> Result<NonZero<HashSet<T>>>
where
    R: Buf,
    T: Hash + Eq,
    D: Fn(&mut bytes::buf::Take<&mut R>) -> Result<T>,
{
    let len = decode_u64(r)?;
    ensure_not_empty!(len);
    ensure_size!(r, len as usize);

    let mut set = HashSet::new();
    let mut r = r.take(len as usize);

    while r.has_remaining() {
        set.insert(decode(&mut r)?);
    }

    Ok(NonZero::<HashSet<_>>::new(set).unwrap())
}

#[inline]
pub fn decode_id(r: &mut impl Buf) -> Result<FieldId> {
    decode_u32(r).and_then(|id| FieldId::from_u32(id).ok_or(Error::UnknownEncoding(id)))
}

#[inline]
pub fn decode_variant_id(r: &mut impl Buf) -> Result<VariantId> {
    decode_u32(r).map(VariantId::new)
}

pub fn decode_skip(r: &mut impl Buf, encoding: FieldEncoding) -> Result<()> {
    match encoding {
        FieldEncoding::Varint => loop {
            ensure_size!(r, 1);
            if r.get_u8() & 0x80 == 0 {
                return Ok(());
            }
        },
        FieldEncoding::LengthPrefixed => {
            let len = decode_u64(r)? as usize;
            ensure_size!(r, len);
            r.advance(len);
            Ok(())
        }
        FieldEncoding::Fixed1 => {
            ensure_size!(r, 1);
            r.advance(1);
            Ok(())
        }
        FieldEncoding::Fixed4 => {
            ensure_size!(r, 4);
            r.advance(4);
            Ok(())
        }
        FieldEncoding::Fixed8 => {
            ensure_size!(r, 8);
            r.advance(8);
            Ok(())
        }
    }
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

impl Decode for String {
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_string(r)
    }
}

impl Decode for Box<str> {
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_string(r).map(String::into_boxed_str)
    }
}

impl Decode for Box<[u8]> {
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_bytes_std(r).map(Vec::into_boxed_slice)
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_vec(r, |r| T::decode(r))
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Hash + Eq + Decode,
    V: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_hash_map(r, |r| K::decode(r), |r| V::decode(r))
    }
}

impl<T> Decode for HashSet<T>
where
    T: Hash + Eq + Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_hash_set(r, |r| T::decode(r))
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_option(r, T::decode)
    }
}

impl<const N: usize, T> Decode for [T; N]
where
    T: Debug + Decode,
{
    #[inline(always)]
    fn decode(r: &mut impl Buf) -> Result<Self> {
        decode_array(r, |r| T::decode(r))
    }
}

impl<T> Decode for std::borrow::Cow<'_, T>
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
