use ethereum_types::U256;
use signatory;
use signatory::ed25519::Signature;
use signatory::signature::Signer as _;
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::{Digest, HashState, MakeDigest};
use crate::state::NetworkState;

struct ValueTransaction {
    to: U256,
}

struct ContractTransaction {
    bytecode: Vec<u8>,
}

enum TransactionKind {
    ValueTransaction(ValueTransaction),
    ContractTransaction(ContractTransaction),
}

pub struct RawTransaction {
    value: u128,
    nonce: u128,
    from: U256,
    kind: TransactionKind,
}

impl MakeDigest for RawTransaction {
    fn make_digest(&self, state: &mut HashState) {
        self.value.make_digest(state);
        self.from.make_digest(state);
        self.nonce.make_digest(state);
        match self.kind {
            TransactionKind::ValueTransaction(ref val) => {
                state.update(&[0]);
                val.to.make_digest(state);
            }
            TransactionKind::ContractTransaction(ref con) => {
                state.update(&[1]);
                state.update(&con.bytecode[..]);
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
        from: U256,
        to: U256,
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
        let mut bytes = [0u8; 32];
        raw.digest().to_little_endian(&mut bytes);
        let signature = signer.sign(&bytes);
        Transaction { raw, signature }
    }

    pub(crate) fn deploy_contract(
        from: U256,
        value: u128,
        nonce: u128,
        bytecode: Vec<u8>,
        signer: &Signer,
    ) -> (Self, U256) {
        // the contract id is the hash of the bytecode xor the author
        let contract_id = (&bytecode[..]).digest() ^ from;

        let kind = TransactionKind::ContractTransaction(ContractTransaction {
            bytecode,
        });
        let raw = RawTransaction {
            from,
            value,
            kind,
            nonce,
        };
        let mut bytes = [0u8; 32];
        raw.digest().to_little_endian(&mut bytes);
        let signature = signer.sign(&bytes);

        (Transaction { raw, signature }, contract_id)
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
                ref bytecode,
            }) => {
                //
                state.deploy_bytecode(bytecode);
            }
        }
    }
}
