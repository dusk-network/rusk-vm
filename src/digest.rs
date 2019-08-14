use std::io::Write;
use std::ptr;

use blake2_rfc::blake2b::Blake2b;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Digest([u8; 32]);

impl Digest {
    pub unsafe fn from_ptr(src: &u8) -> Self {
        let mut arr = [0u8; 32];
        ptr::copy_nonoverlapping(src, &mut arr[0], 1);
        Digest(arr)
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub trait CryptoHash {
    fn crypto_hash(&self, state: &mut HashState);
}

pub trait MakeDigest: CryptoHash {
    fn digest(&self) -> Digest;
}

impl<T: CryptoHash> MakeDigest for T {
    fn digest(&self) -> Digest {
        let mut state = HashState::new();
        self.crypto_hash(&mut state);
        state.fin()
    }
}

pub struct HashState(Blake2b);

impl HashState {
    fn new() -> Self {
        HashState(Blake2b::new(32))
    }

    pub fn fin(self) -> Digest {
        let mut digest = [0u8; 32];
        digest
            .as_mut()
            .write(self.0.finalize().as_bytes())
            .expect("in-memory write");
        Digest(digest)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

impl CryptoHash for signatory::ed25519::PublicKey {
    fn crypto_hash(&self, state: &mut HashState) {
        state.update(self.as_bytes())
    }
}
