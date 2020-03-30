#![no_std]
use dusk_abi::{self, ContractCall, Signature};
use phoenix_abi::{Note, Nullifier};

pub enum StakingCall {
    Stake {
        nullifiers: [Nullifier; Nullifier::MAX],
        notes: [Note; Note::MAX],
        proof: Proof,
        identity: [u8; 32],
        expiration: u64,
    },
    Withdraw {
        note: Note,
        proof: Proof,
    },
    Slash {
        identity: [u8; 32],
        height: u64,
        step: u8,
        sig1: Signature,
        sig2: Signature,
        note: Note,
    },
}

#[no_mangle]
pub fn call() {
    let data: StakingCall = dusk_abi::args();

    match data {
        StakingCall::Stake {
            nullifiers,
            notes,
            proof,
            identity,
            expiration,
        } => {}
        StakingCall::Withdraw { note, proof } => {}
        StakingCall::Slash {
            identity,
            height,
            step,
            sig1,
            sig2,
            note,
        } => {}
    }
}
