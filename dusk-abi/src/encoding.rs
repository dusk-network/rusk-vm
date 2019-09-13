pub use serde::{Deserialize, Serialize};

pub fn serialize<T: Serialize>(t: T, into: &mut [u8]) {
    unimplemented!()
}

pub fn deserialize<'de, T: Deserialize<'de>>(from: &[u8]) -> T {
    unimplemented!()
}
