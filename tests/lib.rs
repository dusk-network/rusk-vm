mod contracts;
mod helpers;

use kelvin::Blake2b;
use std::fs;

use dusk_abi::{ContractCall, FeeCall, Provisioners, Signature, TransferCall};
use phoenix_abi::{Input, Note, Proof, PublicKey};
use rusk_vm::{Contract, GasMeter, NetworkState, Schedule, StandardABI};

#[test]
fn factorial() {
    use factorial::factorial;

    fn factorial_reference(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            n * factorial_reference(n - 1)
        }
    }

    let code = contract_code!("factorial");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 6;
    assert_eq!(
        network
            .call_contract(&contract_id, factorial(n), &mut gas)
            .unwrap(),
        factorial_reference(n)
    );
}

#[test]
#[ignore]
// Keep the tests for reference at the moment, but skip it because they're
// outdated.
//
// To test the `transfer` contract run the tests from `rusk` repo.
//
// Eventually all the contracts will be moved under `rusk`.
// See: https://github.com/dusk-network/rusk/issues/8
fn transfer() {
    let code = contract_code!("transfer");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // Generate some items
    let inputs = [Input::default(); Input::MAX];
    let notes = [Note::default(); Note::MAX];
    let proof = Proof::default();

    let succeeded: bool = network
        .call_contract(
            &contract_id,
            ContractCall::new(TransferCall::Transfer {
                inputs,
                notes,
                proof,
            })
            .unwrap(),
            &mut gas,
        )
        .unwrap();

    // TODO: change condition once Transfer Contract properly implemented
    assert!(
        !succeeded,
        "Transfer Contract called.
         Expected to returns `false` until proper implementation."
    );
    if succeeded {
        // Ensure data was written
        fs::metadata("/tmp/rusk-vm-demo/data").unwrap();

        // Clean up
        fs::remove_dir_all("/tmp/rusk-vm-demo").unwrap();
    }
}

#[test]
#[ignore]
// Keep the tests for reference at the moment, but skip it because they're
// outdated.
// Eventually all the contracts will be moved under `rusk`.
// See: https://github.com/dusk-network/rusk/issues/8
fn fee() {
    let code = contract_code!("fee");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let addresses = Provisioners::default();
    addresses.to_bytes()[0] = 1u8;

    let call: ContractCall<()> = ContractCall::new(FeeCall::Distribute {
        total_reward: 100,
        addresses: addresses,
        pk: PublicKey::default(),
    })
    .unwrap();

    network.call_contract(&contract_id, call, &mut gas).unwrap();

    let mut address = [0u8; 32];
    address[0] = 1u8;

    let call: ContractCall<()> = ContractCall::new(FeeCall::Withdraw {
        sig: Signature::from_slice(&[0u8; 64]),
        address: address,
        value: 50,
        pk: PublicKey::default(),
    })
    .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network.call_contract(&contract_id, call, &mut gas).unwrap();
}
