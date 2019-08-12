use signatory;
use signatory::ed25519::Signature;
use signatory::signature::Signer as _;
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::{CryptoHash, Digest, HashState, MakeDigest};
use crate::state::NetworkState;

struct ValueTransaction {
    from: Digest,
    to: Digest,
    nonce: u128,
}

struct ContractTransaction {
    to: Digest,
    nonce: u128,
}

enum TransactionKind {
    ValueTransaction(ValueTransaction),
    ContractTransaction(ContractTransaction),
}

pub struct RawTransaction {
    value: u128,
    kind: TransactionKind,
}

impl CryptoHash for RawTransaction {
    fn crypto_hash(&self, state: &mut HashState) {
        state.update(&self.value.to_be_bytes());
        match self.kind {
            TransactionKind::ValueTransaction(ref val) => {
                state.update(val.to.as_ref());
                state.update(&val.nonce.to_be_bytes());
            }
            _ => unimplemented!(),
        }
    }
}

pub struct Transaction {
    raw: RawTransaction,
    signature: Signature,
}

impl Transaction {
    pub(crate) fn send_value(
        from: Digest,
        to: Digest,
        value: u128,
        nonce: u128,
        signer: &Signer,
    ) -> Self {
        let kind = TransactionKind::ValueTransaction(ValueTransaction {
            from,
            to,
            nonce,
        });
        let raw = RawTransaction { value, kind };
        let digest = raw.digest();
        let signature = signer.sign(digest.as_ref());
        Transaction { raw, signature }
    }

    pub(crate) fn valid(&self, _state: &NetworkState) -> bool {
        // FIXME
        true
    }

    pub(crate) fn apply(&self, state: &mut NetworkState) {
        let value = self.raw.value;
        match self.raw.kind {
            TransactionKind::ValueTransaction(ValueTransaction {
                from,
                to,
                nonce,
            }) => {
                let from = state.get_account_mut(&from);
                *from.nonce_mut() = nonce;
                *from.balance_mut() -= value;
                *state.get_account_mut(&to).balance_mut() += value
            }
            _ => unimplemented!(),
        }
    }
}
