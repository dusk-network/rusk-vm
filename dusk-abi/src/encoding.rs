use fermion::{self, Error};
use serde::{Deserialize, Serialize};

pub fn encode<T: Serialize>(t: &T, into: &mut [u8]) -> Result<(), Error> {
    fermion::encode(t, into)
}

pub fn decode<'a, T: Deserialize<'a>>(from: &'a [u8]) -> Result<T, Error> {
    fermion::decode(from)
}
