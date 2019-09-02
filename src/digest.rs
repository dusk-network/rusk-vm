use std::io::Write;

use blake2_rfc::blake2b::Blake2b;
use ethereum_types::U256;

pub trait MakeDigest {
    fn make_digest(&self, state: &mut HashState);
}

pub trait Digest: MakeDigest {
    fn digest(&self) -> U256;
}

impl<T: MakeDigest> Digest for T {
    fn digest(&self) -> U256 {
        let mut state = HashState::new();
        self.make_digest(&mut state);
        state.fin()
    }
}

impl MakeDigest for u128 {
    fn make_digest(&self, state: &mut HashState) {
        state.update(&self.to_le_bytes())
    }
}

impl MakeDigest for U256 {
    fn make_digest(&self, state: &mut HashState) {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        state.update(&bytes)
    }
}

impl<'a> MakeDigest for &'a [u8] {
    fn make_digest(&self, state: &mut HashState) {
        state.update(self)
    }
}

pub struct HashState(Blake2b);

impl HashState {
    fn new() -> Self {
        HashState(Blake2b::new(32))
    }

    pub fn fin(self) -> U256 {
        let mut arr = [0u8; 32];
        arr.as_mut()
            .write(self.0.finalize().as_bytes())
            .expect("in-memory write");
        U256::from_little_endian(&arr)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

impl MakeDigest for signatory::ed25519::PublicKey {
    fn make_digest(&self, state: &mut HashState) {
        state.update(self.as_bytes())
    }
}
