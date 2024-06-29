//! Encoding and decoding for variable integers.

use thiserror::Error;

macro_rules! zigzag {
    ($from:ty, $to:ty) => {
        paste::paste! {
            #[doc = "Use the _ZigZag_ scheme to encode an `" $from "` as `" $to "`."]
            #[allow(clippy::cast_sign_loss)]
            #[inline]
            const fn [<zigzag_encode_ $from>](value: $from) -> $to {
                ((value << 1) ^ (value >> ($from::BITS - 1))) as $to
            }

            #[doc = "Convert a _ZigZag_ encoded `" $from "` back to its original data."]
            #[allow(clippy::cast_possible_wrap)]
            #[inline]
            const fn [<zigzag_decode_ $from>](value: $to) -> $from {
                ((value >> 1) as $from) ^ (-((value & 0b1) as $from))
            }
        }
    };
    ($($from:ty => $to:ty),+ $(,)?) => {
        $(zigzag!($from, $to);)+

        #[cfg(test)]
        mod zigzag_tests {
            use super::*;

            paste::paste! {$(
                #[test]
                fn [<roundtrip_ $from>]() {
                    for value in [0, $from::MIN, $from::MAX] {
                        let unsigned = [<zigzag_encode_ $from>](value);
                        let result = [<zigzag_decode_ $from>](unsigned);
                        assert_eq!(value, result);
                    }
                }
            )+}
        }
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
const fn max_size<T>() -> usize {
    (std::mem::size_of::<T>() * 8 + 6) / 7
}

#[inline]
const fn size<T>(leading_zeros: usize) -> usize {
    max(1, (std::mem::size_of::<T>() * 8 - leading_zeros + 6) / 7)
}

#[inline]
const fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

macro_rules! varint {
    ($ty:ty, $signed:ty) => {
        paste::paste! {
            #[doc = "Encode a `" $ty "` as _Varint_."]
            #[inline]
            #[must_use]
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

            #[doc = "Decode a _Varint_ back to a `" $ty "`."]
            ///
            /// # Errors
            ///
            /// Will return `Err` if the raw bytes don't contain an end marker within the possible
            /// maximum byte count valid for the integer.
            #[inline]
            pub fn [<decode_ $ty>](buf: &[u8]) -> Result<($ty, usize), DecodeIntError> {
                let mut value = 0;
                for (i, b) in buf.iter().copied().enumerate().take(max_size::<$ty>()) {
                    value |= $ty::from(b & 0x7f) << (7 * i);

                    if b & 0x80 == 0 {
                        return Ok((value, i + 1));
                    }
                }

                Err(DecodeIntError)
            }

            #[doc = "Calculate the byte size of a `" $ty "` encoded as _Varint_."]
            #[inline]
            #[must_use]
            pub const fn [<size_ $ty>](value: $ty) -> usize {
                size::<$ty>(value.leading_zeros() as usize)
            }

            #[doc = "Encode a `" $signed "` as _Varint_."]
            #[inline]
            #[must_use]
            pub fn [<encode_ $signed>](value: $signed) -> ([u8; max_size::<$ty>()], usize) {
                [<encode_ $ty>]([<zigzag_encode_ $signed>](value))
            }

            #[doc = "Decode a _Varint_ back to a `" $signed "`."]
            ///
            /// # Errors
            ///
            /// Will return `Err` if the raw bytes don't contain an end marker within the possible
            /// maximum byte count valid for the integer.
            #[inline]
            pub fn [<decode_ $signed>](buf: &[u8]) -> Result<($signed, usize), DecodeIntError> {
                [<decode_ $ty>](buf).map(|(v, b)| ([<zigzag_decode_ $signed>](v), b))
            }

            #[doc = "Calculate the byte size of a `" $signed "` encoded as _Varint_."]
            #[inline]
            #[must_use]
            pub const fn [<size_ $signed>](value: $signed) -> usize {
                size::<$ty>([<zigzag_encode_ $signed>](value).leading_zeros() as usize)
            }

        }
    };
    ($(($ty:ty, $signed:ty)),+ $(,)?) => {
        $(varint!($ty, $signed);)+

        #[cfg(test)]
        mod varint_tests {
            use super::*;

            paste::paste! {$(
                #[test]
                fn [<roundtrip_ $ty>]() {
                    for value in [$ty::MIN, 1, $ty::MAX] {
                        let (buf, size) = [<encode_ $ty>](value);
                        let (result, _) = [<decode_ $ty>](&buf[..size]).unwrap();
                        assert_eq!(value, result);
                    }
                }

                #[test]
                fn [<roundtrip_ $signed>]() {
                    for value in [$signed::MIN, -1, 0, 1, $signed::MAX] {
                        let (buf, size) = [<encode_ $signed>](value);
                        let (result, _) = [<decode_ $signed>](&buf[..size]).unwrap();
                        assert_eq!(value, result);
                    }
                }

                #[test]
                fn [<sizecheck_ $ty>]() {
                    assert_eq!(1, [<size_ $ty>]($ty::MIN));
                    assert_eq!(1, [<size_ $ty>](1));
                    assert_eq!(max_size::<$ty>(), [<size_ $ty>]($ty::MAX));
                }

                #[test]
                fn [<sizecheck_ $signed>]() {
                    assert_eq!(max_size::<$signed>(), [<size_ $signed>]($signed::MIN));
                    assert_eq!(1, [<size_ $signed>](-1));
                    assert_eq!(1, [<size_ $signed>](0));
                    assert_eq!(1, [<size_ $signed>](1));
                    assert_eq!(max_size::<$signed>(), [<size_ $signed>]($signed::MAX));
                }
            )+}
        }
    }
}

varint!((u16, i16), (u32, i32), (u64, i64), (u128, i128));

/// Error that can happen when trying to decode a _Varint_ back into a regular integer.
#[derive(Debug, Error)]
#[error("input was lacking a final marker for the end of the integer data")]
pub struct DecodeIntError;

#[cfg(test)]
mod tests {
    #[test]
    fn max_sizes() {
        assert_eq!(3, super::max_size::<u16>());
        assert_eq!(5, super::max_size::<u32>());
        assert_eq!(10, super::max_size::<u64>());
        assert_eq!(19, super::max_size::<u128>());
    }
}
