use core::hash::Hash;
use std::collections::hash_map::DefaultHasher;

use bytecheck::CheckBytes;
use dusk_hamt::{Hamt, Lookup};
use microkelvin::{BranchRef, BranchRefMut, OffsetLen, StoreRef};
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Serialize,
};

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct Map<K, V> {
    wrapping: Hamt<K, V, (), OffsetLen>,
}

impl<K, V> Map<K, V>
where
    K: Archive<Archived = K>
        + Clone
        + Hash
        + Eq
        + for<'a> CheckBytes<DefaultValidator<'a>>,
    K: Deserialize<K, StoreRef<OffsetLen>>,
    V: Archive + Clone,
    V::Archived: Deserialize<V, StoreRef<OffsetLen>>
        + for<'a> CheckBytes<DefaultValidator<'a>>,
{
    pub fn new() -> Self {
        Map {
            wrapping: Hamt::new(),
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        self.wrapping.insert(key, val)
    }

    pub fn get(&self, key: &K) -> Option<impl BranchRef<V>> {
        self.wrapping.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<impl BranchRefMut<V>> {
        self.wrapping.get_mut(key)
    }
}

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
