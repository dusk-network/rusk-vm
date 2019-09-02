use ethereum_types::U256;
use failure::Error;
use signatory;
use signatory::ed25519::Signature;
use signatory::signature::Signer as _;
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::{Digest, HashState, MakeDigest};
use crate::state::NetworkState;

#[derive(Debug)]
struct ValueTransaction {
    to: U256,
}

#[derive(Debug)]
struct DeployContract {
    contract_id: U256,
    bytecode: Vec<u8>,
}

#[derive(Debug)]
struct ContractCall {
    contract_id: U256,
    data: Vec<u8>,
}

#[derive(Debug)]
enum TransactionKind {
    ValueTransaction(ValueTransaction),
    DeployContract(DeployContract),
    ContractCall(ContractCall),
}

#[derive(Debug)]
pub struct RawTransaction {
    value: u128,
    nonce: u128,
    from: U256,
    kind: TransactionKind,
}

impl RawTransaction {
    fn finalize(self, signer: &Signer) -> Transaction {
        let mut bytes = [0u8; 32];
        self.digest().to_little_endian(&mut bytes);
        let signature = signer.sign(&bytes);
        Transaction {
            raw: self,
            signature,
        }
    }
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
            TransactionKind::DeployContract(ref deploy) => {
                state.update(&[1]);
                state.update(&deploy.bytecode[..]);
            }
            TransactionKind::ContractCall(ref call) => {
                state.update(&[2]);
                state.update(&call.data[..]);
            }
        }
    }
}

#[derive(Debug)]
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
        raw.finalize(signer)
    }

    pub(crate) fn call_contract(
        from: U256,
        contract_id: U256,
        nonce: u128,
        value: u128,
        data: Vec<u8>,
        signer: &Signer,
    ) -> Self {
        let kind =
            TransactionKind::ContractCall(ContractCall { contract_id, data });
        let raw = RawTransaction {
            from,
            value,
            nonce,
            kind,
        };
        raw.finalize(signer)
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

        let kind = TransactionKind::DeployContract(DeployContract {
            bytecode,
            contract_id,
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

    pub(crate) fn apply(&self, state: &mut NetworkState) -> Result<(), Error> {
        let raw = &self.raw;
        let source_account = state.get_account_mut(&raw.from);
        *source_account.nonce_mut() = raw.nonce;
        *source_account.balance_mut() -= raw.value;
        match &raw.kind {
            TransactionKind::ValueTransaction(ValueTransaction { to }) => {
                *state.get_account_mut(&to).balance_mut() += raw.value;
            }
            TransactionKind::DeployContract(DeployContract {
                bytecode,
                contract_id,
            }) => {
                state.deploy_contract(bytecode, contract_id, raw.value)?;
            }
            TransactionKind::ContractCall(ContractCall {
                data,
                contract_id,
            }) => {
                state.call_contract(&data, &contract_id, raw.value)?;
            }
        }
        Ok(())
    }
}
