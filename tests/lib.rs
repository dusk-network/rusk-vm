mod contracts;
mod helpers;

use contracts::default_account::DefaultAccount;
use kelvin::{Blake2b, Store};
use tempfile::tempdir;

use rusk_vm::{Digest, GasMeter, NetworkState, Schedule, StandardABI, Wallet};

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

    // let contract_id = network.deploy(code);

    // let n = 6;
    // assert_eq!(
    //     network
    //         .call_contract(genesis_id, factorial(n), &mut gas_meter)
    //         .unwrap(),
    //     factorial_reference(n)
    // );
}
