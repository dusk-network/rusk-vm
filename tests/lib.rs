mod contracts;
mod helpers;

use kelvin::Blake2b;
use std::fs;

use dusk_abi::ContractCall;
use phoenix_abi::{Note, Nullifier};
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
