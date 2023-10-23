//! Format en- and decoding on in-memory data buffers.

pub use decode::*;
pub use encode::*;

mod decode;
mod encode;

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[test]
    fn non_zero_string_valid() {
        let mut buf = Vec::new();
        encode_string(&mut buf, "test");
        assert!(decode_non_zero_string(&mut &*buf).is_ok());
    }

    #[test]
    fn non_zero_string_invalid() {
        let mut buf = Vec::new();
        encode_string(&mut buf, "");
        assert!(matches!(
            decode_non_zero_string(&mut &*buf),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_bytes_valid() {
        let mut buf = Vec::new();
        encode_bytes(&mut buf, &[1, 2, 3]);
        assert!(decode_non_zero_bytes(&mut &*buf).is_ok());
    }

    #[test]
    fn non_zero_bytes_invalid() {
        let mut buf = Vec::new();
        encode_bytes(&mut buf, &[]);
        assert!(matches!(
            decode_non_zero_bytes(&mut &*buf),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_vec_valid() {
        let mut buf = Vec::new();
        encode_vec(&mut buf, &[1, 2, 3], |w, v| encode_u32(w, *v));
        assert!(decode_non_zero_vec(&mut &*buf, decode_u32).is_ok());
    }

    #[test]
    fn non_zero_vec_invalid() {
        let mut buf = Vec::new();
        encode_vec(&mut buf, &[], |w, v| encode_u32(w, *v));
        assert!(matches!(
            decode_non_zero_vec(&mut &*buf, decode_u32),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_hash_map_valid() {
        let mut buf = Vec::new();
        encode_hash_map(
            &mut buf,
            &HashMap::from_iter([(1, true), (2, false)]),
            |w, k| encode_u32(w, *k),
            |w, v| encode_bool(w, *v),
        );
        assert!(decode_non_zero_hash_map(&mut &*buf, decode_u32, decode_bool).is_ok());
    }

    #[test]
    fn non_zero_hash_map_invalid() {
        let mut buf = Vec::new();
        encode_hash_map(
            &mut buf,
            &HashMap::new(),
            |w, k| encode_u32(w, *k),
            |w, v| encode_bool(w, *v),
        );
        assert!(matches!(
            decode_non_zero_hash_map(&mut &*buf, decode_u32, decode_bool),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_hash_set_valid() {
        let mut buf = Vec::new();
        encode_hash_set(&mut buf, &HashSet::from_iter([1, 2, 3]), |w, v| {
            encode_u32(w, *v);
        });
        assert!(decode_non_zero_hash_set(&mut &*buf, decode_u32).is_ok());
    }

    #[test]
    fn non_zero_hash_set_invalid() {
        let mut buf = Vec::new();
        encode_hash_set(&mut buf, &HashSet::new(), |w, v| encode_u32(w, *v));
        assert!(matches!(
            decode_non_zero_hash_set(&mut &*buf, decode_u32),
            Err(Error::Zero),
        ));
    }
}
