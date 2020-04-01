#![no_std]
use dusk_abi::{self, ContractCall, Signature};
use phoenix_abi::{Note, Nullifier, PublicKey};

const TRANSFER_CONTRACT: [u8; 1] = [0u8];

pub enum StakingCall {
    Stake {
        nullifiers: [Nullifier; Nullifier::MAX],
        notes: Note,
        proof: Proof,
        pk: [u8; 32],
        pk_bls: [u8; 32],
        expiration: u64,
    },
    Withdraw {
        note: Note,
        proof: Proof,
        pk: [u8; 32],
        sig: [u8; 32],
    },
    Slash {
        pk: [u8; 32],
        height: u64,
        step: u8,
        sig1: Signature,
        sig2: Signature,
        msg1: [u8; 32],
        msg2: [u8; 32],
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
            pk,
            pk_bls,
            expiration,
        } => {
            (_value, _pk_bls, _deposit_height, expiry_height) =
                dusk_abi::get_storage(identity);
            if expiry_height > current_height {
                panic!("already an active stake for this identity");
            }

            // TODO: add stake maturity rate to current height
            if expiration > current_height {
                panic!("the stake expiry height is too low");
            }

            if !phoenix_abi::is_transparent(notes) {
                panic!("transaction contains obfuscated notes");
            }

            // Since the notes are transparent, we can easily verify
            // if they are being sent to the right address
            // TODO: missing parameter, public key of the contract needs to be
            // devised somehow
            if !phoenix_abi::is_addressed_to(notes) {
                panic!("notes are not addressed to the staking contract");
            }

            // Transfer the given notes to the staking contract
            let call: ContractCall<(
                [Nullifier; Nullifier::MAX],
                [Note; Note::MAX],
            )> = ContractCall::new(nullifiers, notes);
            let address: H256 = dusk_abi::get_storage(&TRANSFER_CONTRACT);
            if !dusk_abi::call_contract(&address, 0, &call) {
                panic!("could not transfer notes");
            }

            // Add the staker to the list
            dusk_abi::set_storage(
                identity,
                (note.value, pk_bls, current_height, expiration),
            );
            dusk_abi::ret(true);
        }
        StakingCall::Withdraw {
            note,
            proof,
            pk,
            sig,
        } => {
            (_value, _pk_bls, deposit_height, expiry_height) =
                dusk_abi::get_storage(identity);
            if expiry_height > current_height {
                panic!("stake is still active for this identity");
            }

            let mut verify_buf = [0u8; 32 + 8];
            let encoded =
                encoding::encode(&(pk, deposit_height), &mut verify_buf)
                    .unwrap();
            if !dusk_abi::verify_ed25519_signature(&pk, &sig, encoded) {
                panic!("invalid signature");
            }

            // call transferfrom

            dusk_abi::ret(true);
        }
        StakingCall::Slash {
            pk,
            height,
            step,
            sig1,
            sig2,
            msg1,
            msg2,
        } => {
            (value, pk_bls, _deposit_height, expiry_height) =
                dusk_abi::get_storage(pk);
            if value == 0 {
                panic!("trying to slash a non-existant staker");
            }

            // Ensure the messages and signatures are correct
            // TODO: bls_verify is not actually using a BLS signature scheme.
            // This should be properly updated when Rusk integrates with
            // dusk-blockchain.
            let mut verify_buf = [0u8; 32 + 8 + 1];
            let encoded =
                encoding::encode(&(msg1, height, step), &mut verify_buf)
                    .unwrap();
            if !dusk_abi::bls_verify(&pk_bls, &sig1, encoded) {
                panic!("invalid sig1");
            }

            let mut verify_buf = [0u8; 32 + 8 + 1];
            let encoded =
                encoding::encode(&(msg2, height, step), &mut verify_buf)
                    .unwrap();
            if !dusk_abi::bls_verify(&pk_bls, &sig2, encoded) {
                panic!("invalid sig2");
            }

            // Remove staker from the list.
            // TODO: the funds are simply locked up right now, but
            // something should happen with them. This should be
            // adjusted once a proper procedure has been devised.
            dusk_abi::set_storage(pk, (0, [0u8; 32], 0, 0));
            dusk_abi::ret(true);
        }
    }
}

pub fn deploy(address: H256) {
    // Set transfer contract address, for later reference.
    dusk_abi::set_storage(&TRANSFER_CONTRACT, address);
}