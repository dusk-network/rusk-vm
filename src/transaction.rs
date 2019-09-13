use ethereum_types::U256;
use failure::Error;
use signatory;
use signatory::ed25519::Signature;
use signatory::signature::Signer as _;
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::{HashState, MakeDigest};
use crate::state::NetworkState;

#[derive(Debug, Clone)]
pub struct Transaction {
    target: U256,
    data: Vec<u8>,
}

impl Transaction {
    // pub(crate) fn call_contract(
    //     target: U256,
    //     contract_id: U256,
    //     nonce: u128,
    //     value: u128,
    //     data: Vec<u8>,
    //     signer: &Signer,
    // ) -> Self {
    //     let raw = Transaction {
    //         target,
    //         value,
    //         nonce,
    //         data,
    //     };
    //     raw.finalize(signer)
    // }

    // pub(crate) fn deploy_contract(
    //     from: U256,
    //     value: u128,
    //     nonce: u128,
    //     bytecode: Vec<u8>,
    //     signer: &Signer,
    // ) -> (Self, U256) {
    //     unimplemented!()

    //     // // the contract id is the hash of the bytecode xor the author
    //     // let contract_id = (&bytecode[..]).digest() ^ from;

    //     // let kind = TransactionKind::DeployContract(DeployContract {
    //     //     bytecode,
    //     //     contract_id,
    //     // });
    //     // let raw = Transaction {
    //     //     from,
    //     //     value,
    //     //     nonce,
    //     //     bytecode,
    //     // };
    //     // let mut bytes = [0u8; 32];
    //     // raw.digest().to_little_endian(&mut bytes);
    //     // let signature = signer.sign(&bytes);

    //     // (Transaction { raw, signature }, contract_id)
    // }

    pub(crate) fn apply(&self, state: &mut NetworkState) -> Result<(), Error> {
        unimplemented!()
        // let raw = &self.raw;
        // let source_contract = state.get_contract_mut(&raw.from);
        // *source_account.nonce_mut() = raw.nonce;
        // *source_account.balance_mut() -= raw.value;
        // state.call_contract(raw.from, &raw.data, &contract_id, raw.value)?;
        // Ok(())
    }
}
