use fermion::{self, Error};
use serde::{Deserialize, Serialize};

/// Encode value for ABI
pub fn encode<'se, T: Serialize>(
    t: &T,
    into: &'se mut [u8],
) -> Result<&'se [u8], Error> {
    fermion::encode(t, into)
}

/// Decode value for ABI
pub fn decode<'de, T: Deserialize<'de>>(from: &'de [u8]) -> Result<T, Error> {
    fermion::decode(from)
}
