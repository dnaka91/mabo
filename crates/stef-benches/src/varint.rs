pub mod postcard {
    #[inline]
    const fn max_size<T>() -> usize {
        (std::mem::size_of::<T>() * 8 + 7) / 7
    }

    #[inline]
    pub fn encode(mut value: u128, buf: &mut [u8]) {
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
    pub fn encode_zigzag(value: i128) -> u128 {
        ((value << 1) ^ (value >> 127)) as u128
    }

    #[inline]
    pub fn decode(buf: &[u8]) -> u128 {
        let mut value = 0;
        for (i, b) in buf.iter().copied().enumerate().take(max_size::<u128>()) {
            value |= ((b & 0x7f) as u128) << (7 * i);

            if b & 0x80 == 0 {
                return value;
            }
        }

        panic!("bad input")
    }

    #[inline]
    pub fn decode_i128(buf: &[u8]) -> i128 {
        decode_zigzag(decode(buf))
    }

    #[inline]
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
    pub fn decode_u16(buf: &[u8]) -> u16 {
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
    pub fn decode_i16(buf: &[u8]) -> i16 {
        let value = decode_u16(buf);
        if value % 2 == 0 {
            (value / 2) as i16
        } else {
            !(value / 2) as i16
        }
    }

    #[inline]
    pub fn decode_u32(buf: &[u8]) -> u32 {
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
    pub fn decode_i32(buf: &[u8]) -> i32 {
        let value = decode_u32(buf);
        if value % 2 == 0 {
            (value / 2) as i32
        } else {
            !(value / 2) as i32
        }
    }

    #[inline]
    pub fn decode_u64(buf: &[u8]) -> u64 {
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
    pub fn decode_i64(buf: &[u8]) -> i64 {
        let value = decode_u64(buf);
        if value % 2 == 0 {
            (value / 2) as i64
        } else {
            !(value / 2) as i64
        }
    }

    #[inline]
    pub fn decode_u128(buf: &[u8]) -> u128 {
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
