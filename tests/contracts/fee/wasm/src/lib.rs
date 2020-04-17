#![no_std]
use dusk_abi::{self, FeeCall, Provisioners};

// TODO: phoenix works with u64, but it would be more advisable to work with u128.
#[no_mangle]
pub fn call() {
    let data: FeeCall = dusk_abi::args();
    match data {
        FeeCall::Withdraw {
            sig: _,
            address,
            value,
            pk,
        } => {
            // Ensure provisioner is allowed to withdraw this amount
            let (nonce, current_reward) =
                dusk_abi::get_storage(&address).unwrap_or((0 as u32, 0 as u64));
            if current_reward < value {
                panic!("insufficient funds");
            }

            // Verify provisioner signature
            // let mut verify_buf = [0u8; 4 + 16];
            // let encoded =
            //     encoding::encode(&(nonce, value), &mut verify_buf).unwrap();
            // if !dusk_abi::verify_ed25519_signature(&address, &sig, encoded) {
            //     panic!("invalid signature!");
            // }

            // Remove withdrawn amount from provisioner allotment
            dusk_abi::set_storage(
                &address,
                (nonce + 1, current_reward - value),
            );

            // Credit provisioner with withdrawn value
            if !phoenix_abi::credit(value, &pk) {
                panic!("could not credit provisioner")
            }
        }
        FeeCall::Distribute {
            total_reward,
            addresses,
            pk,
        } => {
            // TODO: Check that calling tx is a coinbase

            // TODO: implement formula for bg reward derivation
            let block_gen_reward = total_reward / 2;

            if !phoenix_abi::credit(block_gen_reward, &pk) {
                panic!("could not credit block generator")
            }

            // Get reward by dividing the total reward minus the block generator
            // reward by the amount of provisioners
            let provisioners_count = {
                let mut count = 0;
                addresses.to_bytes().chunks(32).for_each(|a| {
                    if a != [0u8; 32] {
                        count += 1;
                    }
                });

                count
            };

            // Allocate leftover rewards equally to all provisioners
            if provisioners_count > 0 {
                let reward =
                    (total_reward - block_gen_reward) / provisioners_count;
                allocate_provisioner_rewards(reward, addresses);
            }
        }
        FeeCall::GetBalanceAndNonce { address } => {
            dusk_abi::ret(
                dusk_abi::get_storage(&address).unwrap_or((0 as u32, 0 as u64)),
            );
        }
    }
}

// This function increases the reward amount for each provisioner address
// in `addresses` by `reward`.
fn allocate_provisioner_rewards(reward: u64, addresses: Provisioners) {
    addresses.to_bytes().chunks(32).for_each(|a| {
        if a != [0u8; 32] {
            let (nonce, current_reward) =
                dusk_abi::get_storage(a).unwrap_or((0 as u32, 0 as u64));
            let new_reward = current_reward + reward;
            dusk_abi::set_storage(a, (nonce, new_reward));
        }
    });
}
