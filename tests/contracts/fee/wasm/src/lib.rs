#![no_std]
use dusk_abi::{self, encoding, ContractCall, Signature, CALL_DATA_SIZE, H256};
use phoenix_abi::{Note, Nullifier};
use serde::{Deserialize, Serialize};

const TRANSFER_CONTRACT: [u8; 1] = [0u8];

#[derive(Serialize, Deserialize, Debug)]
pub enum FeeCall {
    Withdraw {
        sig: Signature,
        address: [u8; 32],
        value: u128,
        r: [u8; 32],
        pk: [u8; 32],
    },
    Distribute {
        r: [u8; 32],
        pk: [u8; 32],
    },
    GetBalanceAndNonce {
        address: [u8; 32],
    },
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let data: FeeCall = dusk_abi::call_data(&mut buffer);
    match data {
        FeeCall::Withdraw {
            sig,
            address,
            value,
            r,
            pk,
        } => {
            let (nonce, current_reward) = dusk_abi::get_storage(&address)
                .unwrap_or((0 as u32, 0 as u128));
            if current_reward < value {
                panic!("insufficient funds");
            }

            let mut verify_buf = [0u8; 4 + 16];
            let encoded =
                encoding::encode(&(nonce, value), &mut verify_buf).unwrap();
            if !dusk_abi::verify_ed25519_signature(&address, &sig, encoded) {
                panic!("invalid signature!");
            }

            dusk_abi::set_storage(
                &address,
                (nonce + 1, current_reward - value),
            );

            let note = phoenix_abi::create_note(value, r, address);
            let call = ContractCall::<(
                [Nullifier; Nullifier::MAX],
                [Note; Note::MAX],
            )>::new((
                [Nullifier; Nullifier::MAX],
                notes,
            ))
            .unwrap();

            let transfer_address =
                dusk_abi::get_storage(&TRANSFER_CONTRACT).unwrap();
            dusk_abi::call_contract(&transfer_address, 0, &call);
        }
        FeeCall::Distribute { r, pk } => {
            // TODO: Check that calling tx is a coinbase

            allocate_provisioner_rewards();

            // TODO: implement mint function on transfer contract
            let call = ContractCall::<()>::new_raw(mint);
            let transfer_address =
                dusk_abi::get_storage(&TRANSFER_CONTRACT).unwrap();
            dusk_abi::call_contract(&transfer_address, 0, &call);

            // TODO: implement formula for bg reward derivation
            let block_gen_reward = total_reward / 100;

            let note = phoenix_abi::create_note(block_gen_reward, r, pk);
            let notes = [Note::default(); Note::MAX];
            notes[0] = note;
            let call = ContractCall::<(
                [Nullifier; Nullifier::MAX],
                [Note; Note::MAX],
            )>::new((
                [Nullifier; Nullifier::MAX],
                notes,
            ))
            .unwrap();

            dusk_abi::call_contract(&transfer_address, 0, &call);
        }
        FeeCall::GetBalanceAndNonce { address } => {
            dusk_abi::ret(
                dusk_abi::get_storage(&address)
                    .unwrap_or((0 as u32, 0 as u128)),
            );
        }
    }
}

fn allocate_provisioner_rewards(total_reward: u128, block_height: u64) {
    let h = block_height - 1;
    let cert = dusk_abi::get_cert(h);

    let reward = total_reward / cert.addresses().len();
    cert.addresses().iter().for_each(|a| {
        let (nonce, current_reward) =
            dusk_abi::get_storage(a).unwrap_or((0 as u32, 0 as u128));
        let new_reward = current_reward + reward;
        dusk_abi::set_storage(a, (nonce, new_reward));
    });
}

pub fn deploy(address: H256) {
    // Set transfer contract address, for later reference.
    dusk_abi::set_storage(&TRANSFER_CONTRACT, address);
}
