#![no_std]
use dusk_abi::{
    self, encoding, ContractCall, Signature, StakingCall, TransferCall, H256,
};
use phoenix_abi::{Note, Nullifier, PublicKey};

const TRANSFER_CONTRACT: [u8; 1] = [0u8];
const PUBLIC_KEY: [u8; 1] = [1u8];

#[no_mangle]
pub fn call() {
    let data: StakingCall = dusk_abi::args();

    match data {
        StakingCall::Init { address, pk } => {
            dusk_abi::set_storage(&TRANSFER_CONTRACT, address);
            dusk_abi::set_storage(&PUBLIC_KEY, pk);
        }
        StakingCall::Stake {
            nullifiers,
            notes,
            // proof,
            pk,
            pk_bls,
            expiration,
            value,
            current_height,
        } => {
            let values: Option<(u64, [u8; 32], u64, u64)> =
                dusk_abi::get_storage(&pk.as_bytes()[0..32]);
            if values.is_some() {
                panic!("already an active stake for this identity");
            }

            // TODO: add stake maturity rate to current height
            if expiration < current_height {
                panic!("the stake expiry height is too low");
            }

            // Transfer the given notes to the staking contract
            let contract_pk = dusk_abi::get_storage(&PUBLIC_KEY).unwrap();
            let call: ContractCall<bool> =
                ContractCall::new(TransferCall::Approve {
                    nullifiers,
                    notes,
                    pk: contract_pk,
                    value,
                })
                .unwrap();
            let address: H256 =
                dusk_abi::get_storage(&TRANSFER_CONTRACT).unwrap();
            if !dusk_abi::call_contract(&address, 0, &call) {
                panic!("could not transfer notes");
            }

            // Add the staker to the list
            dusk_abi::set_storage(
                &pk.as_bytes()[0..32],
                (value, pk_bls, current_height, expiration),
            );
            dusk_abi::ret(true);
        }
        StakingCall::Withdraw {
            note,
            // proof,
            pk,
            sig,
            current_height,
        } => {
            let (value, _pk_bls, deposit_height, expiry_height): (
                u64,
                [u8; 32],
                u64,
                u64,
            ) = dusk_abi::get_storage(&pk.as_bytes()[0..32]).unwrap();
            if expiry_height > current_height {
                panic!("stake is still active for this identity");
            }

            // TODO: actually implement this
            // let mut verify_buf = [0u8; 32 + 8];
            // let encoded =
            //     encoding::encode(&(pk, deposit_height), &mut verify_buf)
            //         .unwrap();
            // if !dusk_abi::verify_ed25519_signature(&pk, &sig, encoded) {
            //     panic!("invalid signature");
            // }

            // call transferfrom
            let contract_pk = dusk_abi::get_storage(&PUBLIC_KEY).unwrap();
            let call: ContractCall<bool> =
                ContractCall::new(TransferCall::TransferFrom {
                    sender: contract_pk,
                    recipient: pk,
                    value: value,
                })
                .unwrap();
            let address: H256 =
                dusk_abi::get_storage(&TRANSFER_CONTRACT).unwrap();
            if !dusk_abi::call_contract(&address, 0, &call) {
                panic!("could not refund stake");
            }

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
            let (value, pk_bls, _deposit_height, expiry_height): (
                u64,
                [u8; 32],
                u64,
                u64,
            ) = dusk_abi::get_storage(&pk.as_bytes()[0..32]).unwrap();
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
            dusk_abi::set_storage(&pk.as_bytes()[0..32], (0, [0u8; 32], 0, 0));
            dusk_abi::ret(true);
        }
        StakingCall::GetStake { pk } => {
            let values: (u64, [u8; 32], u64, u64) =
                dusk_abi::get_storage(&pk.as_bytes()[0..32]).unwrap();
            dusk_abi::ret(values);
        }
    }
}

// TODO: can we deploy with parameters like this?
/*
#[no_mangle]
pub fn deploy(address: H256, pk: PublicKey) {
    // Set transfer contract address, for later reference.
    dusk_abi::set_storage(&TRANSFER_CONTRACT, address);
}
*/
