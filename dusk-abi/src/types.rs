use super::impl_serde;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct H256([u8; 32]);

impl H256 {
    pub fn zero() -> Self {
        H256(Default::default())
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for H256 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl core::fmt::Debug for H256 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Digest(")?;
        for i in 0..32 {
            write!(f, "{:02x}", self.0[i])?;
        }
        write!(f, ")")
    }
}

const SIGNATURE_BYTES: usize = 64;

#[repr(C)]
pub struct Signature([u8; SIGNATURE_BYTES]);

impl Signature {
    pub fn new() -> Self {
        Signature([42u8; SIGNATURE_BYTES])
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut buf = [0u8; 64];
        buf.copy_from_slice(slice);
        Signature(buf)
    }

    pub fn as_array(&self) -> &[u8; 64] {
        &self.0
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Signature {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl_serde!(Signature, SIGNATURE_BYTES);

impl core::fmt::Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Signature(")?;
        for i in 0..64 {
            write!(f, "{:02x}", self.0[i])?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
mod content {
    use std::io::Read;

    use kelvin::{ByteHash, Content, Sink, Source};

    use super::H256;
    use std::io::{self, Write};

    impl<H: ByteHash> Content<H> for H256 {
        fn persist(&mut self, sink: &mut Sink<H>) -> io::Result<()> {
            sink.write_all(&self.0)
        }

        fn restore(source: &mut Source<H>) -> io::Result<Self> {
            let mut h = H256::default();
            source.read_exact(h.as_mut())?;
            Ok(h)
        }
    }
}
