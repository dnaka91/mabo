pub mod postcard {
    #[inline]
    const fn max_size<T>() -> usize {
        (std::mem::size_of::<T>() * 8 + 7) / 7
    }

    #[inline]
    pub fn encode(mut value: u128, buf: &mut [u8]) {
        assert!(buf.len() >= max_size::<u128>());
        for b in buf.iter_mut().take(max_size::<u128>()) {
            *b = value.to_le_bytes()[0];
            if value < 128 {
                break;
            }

            *b |= 0x80;
            value >>= 7;
        }
    }

    #[inline]
    pub fn encode_i128(value: i128, buf: &mut [u8]) {
        encode(encode_zigzag(value), buf);
    }

    #[inline]
    #[must_use]
    pub fn encode_zigzag(value: i128) -> u128 {
        ((value << 1) ^ (value >> 127)) as u128
    }

    #[inline]
    #[must_use]
    pub fn decode(buf: &[u8]) -> u128 {
        assert!(buf.len() >= max_size::<u128>());
        let mut value = 0;
        for (i, b) in buf.iter().copied().enumerate().take(max_size::<u128>()) {
            value |= u128::from(b & 0x7f) << (7 * i);

            if b & 0x80 == 0 {
                return value;
            }
        }

        panic!("bad input")
    }

    #[inline]
    #[must_use]
    pub fn decode_i128(buf: &[u8]) -> i128 {
        decode_zigzag(decode(buf))
    }

    #[inline]
    #[must_use]
    pub fn decode_zigzag(value: u128) -> i128 {
        ((value >> 1) as i128) ^ (-((value & 0b1) as i128))
    }

    #[test]
    fn roundtrip() {
        let mut buf = [0; max_size::<u128>()];

        for value in [
            1,
            u8::MAX.into(),
            u16::MAX.into(),
            u32::MAX.into(),
            u64::MAX.into(),
            u128::MAX,
        ] {
            encode(value, &mut buf);
            let output = decode(&buf);
            assert_eq!(value, output);
        }

        for value in [
            -1,
            i8::MIN.into(),
            i16::MIN.into(),
            i32::MIN.into(),
            i64::MIN.into(),
            i128::MIN,
        ] {
            encode_i128(value, &mut buf);
            let output = decode_i128(&buf);
            assert_eq!(value, output);
        }
    }
}

pub mod bincode {
    #[inline]
    pub fn encode_u16(value: u16, buf: &mut [u8]) {
        assert!(buf.len() > std::mem::size_of::<u16>());
        if value <= 250 {
            buf[0] = value.to_le_bytes()[0];
        } else {
            buf[0] = 251;
            buf[1..3].copy_from_slice(&value.to_le_bytes()[..2]);
        }
    }

    #[inline]
    pub fn encode_i16(value: i16, buf: &mut [u8]) {
        encode_u16(
            if value < 0 {
                !(value as u16) * 2 + 1
            } else {
                (value as u16) * 2
            },
            buf,
        );
    }

    #[inline]
    pub fn encode_u32(value: u32, buf: &mut [u8]) {
        assert!(buf.len() > std::mem::size_of::<u32>());
        if value <= 250 {
            buf[0] = value.to_le_bytes()[0];
        } else if value <= u16::MAX.into() {
            buf[0] = 251;
            buf[1..3].copy_from_slice(&value.to_le_bytes()[..2]);
        } else {
            buf[0] = 252;
            buf[1..5].copy_from_slice(&value.to_le_bytes()[..4]);
        }
    }

    #[inline]
    pub fn encode_i32(value: i32, buf: &mut [u8]) {
        encode_u32(
            if value < 0 {
                !(value as u32) * 2 + 1
            } else {
                (value as u32) * 2
            },
            buf,
        );
    }

    #[inline]
    pub fn encode_u64(value: u64, buf: &mut [u8]) {
        assert!(buf.len() > std::mem::size_of::<u64>());
        if value <= 250 {
            buf[0] = value.to_le_bytes()[0];
        } else if value <= u16::MAX.into() {
            buf[0] = 251;
            buf[1..3].copy_from_slice(&value.to_le_bytes()[..2]);
        } else if value <= u32::MAX.into() {
            buf[0] = 252;
            buf[1..5].copy_from_slice(&value.to_le_bytes()[..4]);
        } else {
            buf[0] = 253;
            buf[1..9].copy_from_slice(&value.to_le_bytes()[..8]);
        }
    }

    #[inline]
    pub fn encode_i64(value: i64, buf: &mut [u8]) {
        encode_u64(
            if value < 0 {
                !(value as u64) * 2 + 1
            } else {
                (value as u64) * 2
            },
            buf,
        );
    }

    #[inline]
    pub fn encode_u128(value: u128, buf: &mut [u8]) {
        assert!(buf.len() > std::mem::size_of::<u128>());
        if value <= 250 {
            buf[0] = value.to_le_bytes()[0];
        } else if value <= u16::MAX.into() {
            buf[0] = 251;
            buf[1..3].copy_from_slice(&value.to_le_bytes()[..2]);
        } else if value <= u32::MAX.into() {
            buf[0] = 252;
            buf[1..5].copy_from_slice(&value.to_le_bytes()[..4]);
        } else if value <= u64::MAX.into() {
            buf[0] = 253;
            buf[1..9].copy_from_slice(&value.to_le_bytes()[..8]);
        } else {
            buf[0] = 254;
            buf[1..17].copy_from_slice(&value.to_le_bytes()[..16]);
        }
    }

    #[inline]
    pub fn encode_i128(value: i128, buf: &mut [u8]) {
        encode_u128(
            if value < 0 {
                !(value as u128) * 2 + 1
            } else {
                (value as u128) * 2
            },
            buf,
        );
    }

    #[inline]
    #[must_use]
    pub fn decode_u16(buf: &[u8]) -> u16 {
        assert!(buf.len() > std::mem::size_of::<u16>());
        match buf[0] {
            byte @ 0..=250 => byte.into(),
            251 => {
                let mut b = [0; 2];
                b.copy_from_slice(&buf[1..3]);
                u16::from_le_bytes(b)
            }
            _ => panic!("bad input"),
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_i16(buf: &[u8]) -> i16 {
        let value = decode_u16(buf);
        if value % 2 == 0 {
            (value / 2) as i16
        } else {
            !(value / 2) as i16
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_u32(buf: &[u8]) -> u32 {
        assert!(buf.len() > std::mem::size_of::<u32>());
        match buf[0] {
            byte @ 0..=250 => byte.into(),
            251 => {
                let mut b = [0; 2];
                b.copy_from_slice(&buf[1..3]);
                u16::from_le_bytes(b).into()
            }
            252 => {
                let mut b = [0; 4];
                b.copy_from_slice(&buf[1..5]);
                u32::from_le_bytes(b)
            }
            _ => panic!("bad input"),
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_i32(buf: &[u8]) -> i32 {
        let value = decode_u32(buf);
        if value % 2 == 0 {
            (value / 2) as i32
        } else {
            !(value / 2) as i32
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_u64(buf: &[u8]) -> u64 {
        assert!(buf.len() > std::mem::size_of::<u64>());
        match buf[0] {
            byte @ 0..=250 => byte.into(),
            251 => {
                let mut b = [0; 2];
                b.copy_from_slice(&buf[1..3]);
                u16::from_le_bytes(b).into()
            }
            252 => {
                let mut b = [0; 4];
                b.copy_from_slice(&buf[1..5]);
                u32::from_le_bytes(b).into()
            }
            253 => {
                let mut b = [0; 8];
                b.copy_from_slice(&buf[1..9]);
                u64::from_le_bytes(b)
            }
            _ => panic!("bad input"),
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_i64(buf: &[u8]) -> i64 {
        let value = decode_u64(buf);
        if value % 2 == 0 {
            (value / 2) as i64
        } else {
            !(value / 2) as i64
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_u128(buf: &[u8]) -> u128 {
        assert!(buf.len() > std::mem::size_of::<u128>());
        match buf[0] {
            byte @ 0..=250 => byte.into(),
            251 => {
                let mut b = [0; 2];
                b.copy_from_slice(&buf[1..3]);
                u16::from_le_bytes(b).into()
            }
            252 => {
                let mut b = [0; 4];
                b.copy_from_slice(&buf[1..5]);
                u32::from_le_bytes(b).into()
            }
            253 => {
                let mut b = [0; 8];
                b.copy_from_slice(&buf[1..9]);
                u64::from_le_bytes(b).into()
            }
            254 => {
                let mut b = [0; 16];
                b.copy_from_slice(&buf[1..17]);
                u128::from_le_bytes(b)
            }
            _ => panic!("bad input"),
        }
    }

    #[inline]
    #[must_use]
    pub fn decode_i128(buf: &[u8]) -> i128 {
        let value = decode_u128(buf);
        if value % 2 == 0 {
            (value / 2) as i128
        } else {
            !(value / 2) as i128
        }
    }

    #[test]
    fn roundtrip() {
        let mut buf = [0; 17];

        for value in [
            1,
            u8::MAX.into(),
            u16::MAX.into(),
            u32::MAX.into(),
            u64::MAX.into(),
            u128::MAX,
        ] {
            encode_u128(value, &mut buf);
            let output = decode_u128(&buf);
            assert_eq!(value, output);
        }

        for value in [
            -1,
            i8::MIN.into(),
            i16::MIN.into(),
            i32::MIN.into(),
            i64::MIN.into(),
            i128::MIN,
        ] {
            encode_i128(value, &mut buf);
            let output = decode_i128(&buf);
            assert_eq!(value, output);
        }
    }
}

/// # Vu128: Efficient variable-length integers
///
/// <https://john-millikin.com/vu128-efficient-variable-length-integers>
pub mod vu128 {
    macro_rules! encode {
        ($name:ident, $ty:ident, $len_mask:literal) => {
            #[expect(clippy::cast_possible_truncation)]
            #[inline]
            pub fn $name(value: $ty, buf: &mut [u8]) {
                assert!(buf.len() > std::mem::size_of::<$ty>());
                if value < 0xf0 {
                    buf[0] = value as u8;
                    return;
                }
                buf[1..][..std::mem::size_of::<$ty>()].copy_from_slice(&value.to_le_bytes());
                let len = ((value.leading_zeros() >> 3) as u8) ^ $len_mask;
                buf[0] = 0xf0 | len;
            }
        };
    }

    macro_rules! decode {
        ($name:ident, $ty:ident, $len_mask:literal) => {
            #[inline]
            #[must_use]
            pub fn $name(buf: &[u8]) -> $ty {
                assert!(buf.len() > std::mem::size_of::<$ty>());
                if buf[0] < 0xf0 {
                    return $ty::from(buf[0]);
                }
                let value =
                    $ty::from_le_bytes(buf[1..][..std::mem::size_of::<$ty>()].try_into().unwrap());
                let len = buf[0] & 0x0f;
                let mask = $ty::MAX >> ((len & $len_mask) ^ $len_mask);
                value & mask
            }
        };
    }

    #[inline]
    pub fn encode_u8(value: u8, buf: &mut [u8]) {
        assert!(buf.len() > 1);
        if value < 0xf0 {
            buf[0] = value;
            return;
        }
        buf[0] = 0xf0;
        buf[1] = value;
    }

    encode!(encode_u16, u16, 0x01);
    encode!(encode_u32, u32, 0x03);
    encode!(encode_u64, u64, 0x07);
    encode!(encode_u128, u128, 0x0f);

    #[inline]
    #[must_use]
    pub fn decode_u8(buf: &[u8]) -> u8 {
        assert!(buf.len() > 1);
        if buf[0] < 0x0f {
            return buf[0];
        }
        buf[1]
    }

    decode!(decode_u16, u16, 0x01);
    decode!(decode_u32, u32, 0x03);
    decode!(decode_u64, u64, 0x07);
    decode!(decode_u128, u128, 0x0f);

    macro_rules! encode_i {
        (
            $name:ident,
            $encode_fn:ident,
            $ti:ident,
            $tu:ident,
            $zigzag_shift:literal
        ) => {
            #[inline]
            pub fn $name(value: $ti, buf: &mut [u8]) {
                let zigzag = ((value >> $zigzag_shift) as $tu) ^ ((value << 1) as $tu);
                $encode_fn(zigzag, buf);
            }
        };
    }

    macro_rules! decode_i {
        (
            $name:ident,
            $decode_fn:ident,
            $ti:ident,
            $tu:ident
        ) => {
            #[inline]
            #[must_use]
            pub fn $name(buf: &[u8]) -> $ti {
                let zz = $decode_fn(buf);
                ((zz >> 1) as $ti) ^ (-((zz & 1) as $ti))
            }
        };
    }

    encode_i!(encode_i8, encode_u8, i8, u8, 7);
    encode_i!(encode_i16, encode_u16, i16, u16, 15);
    encode_i!(encode_i32, encode_u32, i32, u32, 31);
    encode_i!(encode_i64, encode_u64, i64, u64, 63);
    encode_i!(encode_i128, encode_u128, i128, u128, 127);

    decode_i!(decode_i8, decode_u8, i8, u8);
    decode_i!(decode_i16, decode_u16, i16, u16);
    decode_i!(decode_i32, decode_u32, i32, u32);
    decode_i!(decode_i64, decode_u64, i64, u64);
    decode_i!(decode_i128, decode_u128, i128, u128);

    #[inline]
    pub fn encode_f32(value: f32, buf: &mut [u8; 5]) {
        encode_u32(value.to_bits().swap_bytes(), buf);
    }

    #[inline]
    pub fn encode_f64(value: f64, buf: &mut [u8; 9]) {
        encode_u64(value.to_bits().swap_bytes(), buf);
    }

    #[inline]
    #[must_use]
    pub fn decode_f32(buf: &[u8]) -> f32 {
        let swapped = decode_u32(buf);
        f32::from_bits(swapped.swap_bytes())
    }

    #[inline]
    #[must_use]
    pub fn decode_f64(buf: &[u8]) -> f64 {
        let swapped = decode_u64(buf);
        f64::from_bits(swapped.swap_bytes())
    }

    #[test]
    fn roundtrip() {
        let mut buf = [0; 17];

        for value in [
            1,
            u8::MAX.into(),
            u16::MAX.into(),
            u32::MAX.into(),
            u64::MAX.into(),
            u128::MAX,
        ] {
            encode_u128(value, &mut buf);
            let output = decode_u128(&buf);
            assert_eq!(value, output);
        }

        for value in [
            -1,
            i8::MIN.into(),
            i16::MIN.into(),
            i32::MIN.into(),
            i64::MIN.into(),
            i128::MIN,
        ] {
            encode_i128(value, &mut buf);
            let output = decode_i128(&buf);
            assert_eq!(value, output);
        }
    }
}
