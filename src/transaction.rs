use signatory;
use signatory::ed25519::Signature;
use signatory::signature::Signer as _;
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::{CryptoHash, Digest, HashState, MakeDigest};
use crate::state::NetworkState;

struct ValueTransaction {
    to: Digest,
}

struct ContractTransaction {
    code: Vec<u8>,
}

enum TransactionKind {
    ValueTransaction(ValueTransaction),
    ContractTransaction(ContractTransaction),
}

pub struct RawTransaction {
    value: u128,
    nonce: u128,
    from: Digest,
    kind: TransactionKind,
}

impl CryptoHash for RawTransaction {
    fn crypto_hash(&self, state: &mut HashState) {
        state.update(&self.value.to_be_bytes());
        state.update(&self.from.as_ref());
        state.update(&self.nonce.to_be_bytes());
        match self.kind {
            TransactionKind::ValueTransaction(ref val) => {
                state.update(&[0]);
                state.update(val.to.as_ref());
            }
            TransactionKind::ContractTransaction(ref con) => {
                state.update(&[1]);
                state.update(&con.code[..]);
            }
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
        let kind = TransactionKind::ValueTransaction(ValueTransaction { to });
        let raw = RawTransaction {
            from,
            value,
            kind,
            nonce,
        };
        let digest = raw.digest();
        let signature = signer.sign(digest.as_ref());
        Transaction { raw, signature }
    }

    pub(crate) fn deploy_contract(
        from: Digest,
        value: u128,
        nonce: u128,
        code: Vec<u8>,
        signer: &Signer,
    ) -> Self {
        let kind =
            TransactionKind::ContractTransaction(ContractTransaction { code });
        let raw = RawTransaction {
            from,
            value,
            kind,
            nonce,
        };
        let digest = raw.digest();
        let signature = signer.sign(digest.as_ref());
        Transaction { raw, signature }
    }

    pub(crate) fn valid(&self, _state: &NetworkState) -> bool {
        // FIXME
        true
    }

    pub(crate) fn apply(&self, state: &mut NetworkState) {
        let raw = &self.raw;
        let source_account = state.get_account_mut(&raw.from);
        *source_account.nonce_mut() = raw.nonce;
        *source_account.balance_mut() -= raw.value;
        match raw.kind {
            TransactionKind::ValueTransaction(ValueTransaction { to }) => {
                *state.get_account_mut(&to).balance_mut() += raw.value;
            }
            TransactionKind::ContractTransaction(ContractTransaction {
                ref code,
            }) => {
                //
                state.deploy_code(code);
            }
        }
    }
}
