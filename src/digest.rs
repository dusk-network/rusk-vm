use std::io::Write;

use blake2_rfc::blake2b::Blake2b;
use dusk_abi::H256;

pub trait MakeDigest {
    fn make_digest(&self, state: &mut HashState);
}

pub trait Digest: MakeDigest {
    fn digest(&self) -> H256;
}

impl<T: MakeDigest> Digest for T {
    fn digest(&self) -> H256 {
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

impl MakeDigest for H256 {
    fn make_digest(&self, state: &mut HashState) {
        state.update(self.as_ref())
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

    pub fn fin(self) -> H256 {
        let mut digest = H256::zero();
        digest
            .as_mut()
            .write_all(self.0.finalize().as_bytes())
            .expect("in-memory write");
        digest
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
