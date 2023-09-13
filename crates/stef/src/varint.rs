//! Encoding and decoding for variable integers.

use thiserror::Error;

macro_rules! zigzag {
    ($from:ty, $to:ty) => {
        paste::paste! {
            #[doc = "Use the _ZigZag_ scheme to encode an `" $from "` as `" $to "`."]
            #[inline]
            fn [<zigzag_encode_ $from>](value: $from) -> $to {
                ((value << 1) ^ (value >> ($from::BITS - 1))) as $to
            }

            #[doc = "Convert a _ZigZag_ encoded `" $from "` back to its original data."]
            #[inline]
            fn [<zigzag_decode_ $from>](value: $to) -> $from {
                ((value >> 1) as $from) ^ (-((value & 0b1) as $from))
            }
        }
    };
    ($($from:ty => $to:ty),+ $(,)?) => {
        $(zigzag!($from, $to);)+
    }
}

zigzag!(
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128,
);

/// Calculate the maximum amount of bytes that an integer might require to be encoded as _varint_.
#[inline]
pub(crate) const fn max_size<T>() -> usize {
    (std::mem::size_of::<T>() * 8 + 7) / 7
}

macro_rules! varint {
    ($ty:ty, $signed:ty) => {
        paste::paste! {
            #[inline]
            pub fn [<encode_ $ty>](mut value: $ty) -> ([u8; max_size::<$ty>()], usize) {
                let mut buf = [0; max_size::<$ty>()];

                for (i, b) in buf.iter_mut().enumerate() {
                    *b = (value & 0xff) as u8;
                    if value < 128 {
                        return (buf, i + 1);
                    }

                    *b |= 0x80;
                    value >>= 7;
                }

                debug_assert_eq!(value, 0);
                (buf, buf.len())
            }

            #[inline]
            pub fn [<decode_ $ty>](buf: &[u8]) -> Result<($ty, usize), DecodeIntError> {
                let mut value = 0;
                for (i, b) in buf.iter().copied().enumerate().take(max_size::<$ty>()) {
                    value |= ((b & 0x7f) as $ty) << (7 * i);

                    if b & 0x80 == 0 {
                        return Ok((value, i + 1));
                    }
                }

                Err(DecodeIntError)
            }

            #[inline]
            pub fn [<encode_ $signed>](value: $signed) -> ([u8; max_size::<$ty>()], usize) {
                [<encode_ $ty>]([<zigzag_encode_ $signed>](value))
            }

            #[inline]
            pub fn [<decode_ $signed>](buf: &[u8]) -> Result<($signed, usize), DecodeIntError> {
                [<decode_ $ty>](buf).map(|(v, b)| ([<zigzag_decode_ $signed>](v), b))
            }
        }
    };
    ($(($ty:ty, $signed:ty)),+ $(,)?) => {
        $(varint!($ty, $signed);)+
    }
}

varint!((u16, i16), (u32, i32), (u64, i64), (u128, i128));

#[derive(Debug, Error)]
#[error("input was lacking a final marker for the end of the integer data")]
pub struct DecodeIntError;
