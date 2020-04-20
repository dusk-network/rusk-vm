mod contracts;
mod helpers;

use kelvin::Blake2b;

use rusk_vm::{Contract, GasMeter, NetworkState, Schedule, StandardABI};

#[test]
fn factorial() {
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

    let n: u64 = 6;

    assert_eq!(
        network
            .call_contract::<u64, u64>(contract_id, n, &mut gas)
            .unwrap(),
        factorial_reference(n)
    );
}

#[test]
fn storage() {
    let code = contract_code!("storage");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let non_existing = network
        .call_contract::<i32, i32>(contract_id, -1, &mut gas)
        .unwrap();

    assert_eq!(non_existing, -1);

    let set = network
        .call_contract::<i32, i32>(contract_id, 42, &mut gas)
        .unwrap();

    assert_eq!(set, 42);

    let delete = network
        .call_contract::<i32, i32>(contract_id, -2, &mut gas)
        .unwrap();

    assert_eq!(delete, -2);

    let non_existing_again = network
        .call_contract::<i32, i32>(contract_id, -1, &mut gas)
        .unwrap();

    assert_eq!(non_existing_again, -1);
}
