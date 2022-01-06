use core::hash::Hash;
use std::collections::hash_map::DefaultHasher;

pub fn hash_mocker(bytes: &[u8]) -> [u8; 32] {
    use std::convert::TryFrom;
    use std::hash::Hasher;
    let mut a: [u8; 32] =
        <[u8; 32]>::try_from(&bytes[bytes.len() - 32..]).unwrap();
    let mut s = DefaultHasher::new();
    bytes.hash(&mut s);
    a[24..].copy_from_slice(&s.finish().to_le_bytes());
    a
}
