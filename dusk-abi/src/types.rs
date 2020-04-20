use dataview::Pod;

/// The standard hash type of 32 bytes
#[derive(Pod, Clone, Copy, Default, PartialEq, Eq)]
pub struct H256 {
    bytes: [u8; 32],
}

impl H256 {
    /// Create a H256 from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 32);
        let mut new = H256::default();
        new.as_mut().copy_from_slice(bytes);
        new
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for H256 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl core::fmt::Debug for H256 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Digest(")?;
        for i in 0..32 {
            write!(f, "{:02x}", self.bytes[i])?;
        }
        write!(f, ")")
    }
}

const SIGNATURE_BYTES: usize = 64;

/// Standard 64 byte signature type
#[repr(C)]
pub struct Signature([u8; SIGNATURE_BYTES]);

impl Signature {
    /// Create a new signature from a byte slice
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut buf = [0u8; 64];
        buf.copy_from_slice(slice);
        Signature(buf)
    }

    /// Returns a reference to the internal byte array
    pub fn as_array_ref(&self) -> &[u8; 64] {
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
            sink.write_all(&self.bytes)
        }

        fn restore(source: &mut Source<H>) -> io::Result<Self> {
            let mut h = H256::default();
            source.read_exact(h.as_mut())?;
            Ok(h)
        }
    }
}
