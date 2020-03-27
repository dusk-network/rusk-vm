mod contracts;
mod helpers;

use kelvin::Blake2b;
use std::fs;

use dusk_abi::{ContractCall, FeeCall, Provisioners, Signature};
use phoenix_abi::{Note, Nullifier, PublicKey};
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
fn hello_world() {
    let code = contract_code!("hello");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .call_contract(&contract_id, ContractCall::<()>::nil(), &mut gas)
        .unwrap();
}

#[test]
fn transfer() {
    use transfer::transfer;

    let code = contract_code!("transfer");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // Generate some items
    let nullifiers = [Nullifier::default(); Nullifier::MAX];
    let notes = [Note::default(); Note::MAX];

    let succeeded = network
        .call_contract(&contract_id, transfer(nullifiers, notes), &mut gas)
        .unwrap();

    // TODO: change condition once Transfer Contract properly implemented
    assert!(
        !succeeded,
        "Transfer Contract called. 
         Expected to returns `false` until proper implementation."
    );
    if (succeeded) {
        // Ensure data was written
        fs::metadata("/tmp/rusk-vm-demo/data").unwrap();

        // Clean up
        fs::remove_dir_all("/tmp/rusk-vm-demo").unwrap();
    }
}

#[test]
fn fee() {
    use fee;

    let code = contract_code!("fee");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let mut addresses = Provisioners::default();
    addresses.0[0] = 1u8;

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
