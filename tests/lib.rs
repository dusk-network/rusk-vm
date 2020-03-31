mod contracts;
mod helpers;

use kelvin::Blake2b;

use dusk_abi::TransferCall;
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
