//! Format en- and decoding on in-memory data buffers.

pub use decode::*;
pub use encode::*;
pub use size::*;

mod decode;
mod encode;
mod size;

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
    fn non_zero_bytes_std_valid() {
        let mut buf = Vec::new();
        encode_bytes_std(&mut buf, &[1, 2, 3]);
        assert!(decode_non_zero_bytes_std(&mut &*buf).is_ok());
    }

    #[test]
    fn non_zero_bytes_std_invalid() {
        let mut buf = Vec::new();
        encode_bytes_std(&mut buf, &[]);
        assert!(matches!(
            decode_non_zero_bytes_std(&mut &*buf),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_bytes_bytes_valid() {
        let mut buf = Vec::new();
        encode_bytes_bytes(&mut buf, &Bytes::from_static(&[1, 2, 3]));
        assert!(decode_non_zero_bytes_bytes(&mut &*buf).is_ok());
    }

    #[test]
    fn non_zero_bytes_bytes_invalid() {
        let mut buf = Vec::new();
        encode_bytes_bytes(&mut buf, &Bytes::from_static(&[]));
        assert!(matches!(
            decode_non_zero_bytes_bytes(&mut &*buf),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_vec_valid() {
        let mut buf = Vec::new();
        encode_vec(
            &mut buf,
            &[1, 2, 3],
            |v| size_u32(*v),
            |w, v| encode_u32(w, *v),
        );
        assert!(decode_non_zero_vec(&mut &*buf, |r| decode_u32(r)).is_ok());
    }

    #[test]
    fn non_zero_vec_invalid() {
        let mut buf = Vec::new();
        encode_vec(&mut buf, &[], |v| size_u32(*v), |w, v| encode_u32(w, *v));
        assert!(matches!(
            decode_non_zero_vec(&mut &*buf, |r| decode_u32(r)),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_hash_map_valid() {
        let mut buf = Vec::new();
        encode_hash_map(
            &mut buf,
            &HashMap::from_iter([(1, true), (2, false)]),
            |k| size_u32(*k),
            |v| size_bool(*v),
            |w, k| encode_u32(w, *k),
            |w, v| encode_bool(w, *v),
        );
        assert!(
            decode_non_zero_hash_map(&mut &*buf, |r| decode_u32(r), |r| decode_bool(r)).is_ok()
        );
    }

    #[test]
    fn non_zero_hash_map_invalid() {
        let mut buf = Vec::new();
        encode_hash_map(
            &mut buf,
            &HashMap::new(),
            |k| size_u32(*k),
            |v| size_bool(*v),
            |w, k| encode_u32(w, *k),
            |w, v| encode_bool(w, *v),
        );
        assert!(matches!(
            decode_non_zero_hash_map(&mut &*buf, |r| decode_u32(r), |r| decode_bool(r)),
            Err(Error::Zero),
        ));
    }

    #[test]
    fn non_zero_hash_set_valid() {
        let mut buf = Vec::new();
        encode_hash_set(
            &mut buf,
            &HashSet::from_iter([1, 2, 3]),
            |v| size_u32(*v),
            |w, v| {
                encode_u32(w, *v);
            },
        );
        assert!(decode_non_zero_hash_set(&mut &*buf, |r| decode_u32(r)).is_ok());
    }

    #[test]
    fn non_zero_hash_set_invalid() {
        let mut buf = Vec::new();
        encode_hash_set(
            &mut buf,
            &HashSet::new(),
            |v| size_u32(*v),
            |w, v| encode_u32(w, *v),
        );
        assert!(matches!(
            decode_non_zero_hash_set(&mut &*buf, |r| decode_u32(r)),
            Err(Error::Zero),
        ));
    }
}
