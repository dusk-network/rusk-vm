use super::impl_serde_for_array;
// use serde::de::Visitor;
// use serde::ser::SerializeTuple;
use super::Provisioners;
use phoenix_abi::types::{Input, Note, Proof, PublicKey};
use serde::{Deserialize, Serialize};

/// The standard hash type of 32 bytes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct H256([u8; 32]);

impl H256 {
    /// Return a zero-hash
    pub fn zero() -> Self {
        H256(Default::default())
    }

    /// Create a H256 from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 32);
        let mut new = H256::zero();
        new.as_mut().copy_from_slice(bytes);
        new
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

impl_serde_for_array!(Signature, SIGNATURE_BYTES);

impl core::fmt::Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Signature(")?;
        for i in 0..64 {
            write!(f, "{:02x}", self.0[i])?;
        }
        Ok(())
    }
}

/// The `TransferCall` type.
/// All the possible methods available for the `Transfer` Genesis Contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum TransferCall {
    /// The `Transfer` call arguments
    Transfer {
        /// The inputs for the Transfer call
        inputs: [Input; Input::MAX],
        /// The notes for the Transfer call
        notes: [Note; Note::MAX],
        /// The proof for the Transfer call
        proof: Proof,
    },
    /// The `Approve` call arguments
    Approve {
        /// The inputs for the Approve call
        inputs: [Input; Input::MAX],
        /// The notes for the Approve call
        notes: [Note; Note::MAX],
        /// The Public Key for the Approve call
        pk: PublicKey,
        /// The transparent value for the Approve call
        value: u64,
        /// The proof for the Approve call
        proof: Proof,
    },
    /// The `TransferFrom` call arguments
    TransferFrom {
        /// The sender's public key used for TransferFrom call
        sender: PublicKey,
        /// The recipient's public key used for TransferFrom call
        recipient: PublicKey,
        /// The transparent value for the TransferFrom call
        value: u64,
    },
}

/// The `FeeCall` type.
/// All the possible methods available for the `Fee` Genesis Contract.
#[derive(Serialize, Deserialize, Debug)]
pub enum FeeCall {
    /// The Withdraw call arguments
    Withdraw {
        /// The signature used for Withdraw call
        sig: Signature,
        /// The address used for Withdraw call
        address: [u8; 32],
        /// The value used for Withdraw call
        value: u64,
        /// The public key used for Withdraw call
        pk: PublicKey,
    },
    /// The Distribute call arguments
    Distribute {
        /// The total reward to distribute
        total_reward: u64,
        /// The list of provisioners
        addresses: Provisioners,
        /// The Public Key of the block generator, to distribute a portion of
        /// the reward to
        pk: PublicKey,
    },
    /// The GetBalanceAndNonce call argument
    GetBalanceAndNonce {
        /// The address to get the balance and nonce from
        address: [u8; 32],
    },
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
