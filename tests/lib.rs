mod contracts;
mod helpers;

use contracts::default_account::DefaultAccount;
use kelvin::{Blake2b, Store};
use tempfile::tempdir;

use rusk_vm::{
    ContractModule, Digest, GasMeter, NetworkState, Schedule, Wallet,
};

#[test]
fn default_account() {
    let mut wallet = Wallet::new();
    let schedule = Schedule::default();
    let mut genesis_builder =
        ContractModule::new(contract_code!("default_account"), &schedule)
            .unwrap();

    let pub_key = wallet.default_account().public_key();
    genesis_builder
        .set_parameter("PUBLIC_KEY", pub_key)
        .unwrap();

    let genesis = genesis_builder.build().unwrap();

    // New genesis network with initial value
    let mut network = NetworkState::genesis(genesis, 1_000_000_000).unwrap();

    let genesis_id = *network.genesis_id();

    // check balance of genesis account
    assert_eq!(
        network
            .call_contract(genesis_id, DefaultAccount::balance())
            .unwrap(),
        1_000_000_000
    );

    // setup a secondary account

    wallet.new_account("alice").unwrap();
    let schedule = Schedule::default();
    let mut account_builder =
        ContractModule::new(contract_code!("default_account"), &schedule)
            .unwrap();

    let alice_pub_key = wallet.get_account("alice").unwrap().public_key();
    account_builder
        .set_parameter("PUBLIC_KEY", alice_pub_key)
        .unwrap();

    let alice_account = account_builder.build().unwrap();
    let alice_account_id = alice_account.digest();
    // transfer 1000 to alice from genesis account

    let genesis_signer = wallet.default_account().signer();

    let call = DefaultAccount::transfer(
        genesis_signer,
        alice_account.digest(),
        1000,
        0,
    );

    network.call_contract(genesis_id, call).unwrap();

    // deploy/reveal alices contract

    network.deploy_contract(alice_account).unwrap();

    // check balances

    assert_eq!(
        network
            .call_contract(alice_account_id, DefaultAccount::balance())
            .unwrap(),
        1_000,
    );

    assert_eq!(
        network
            .call_contract(genesis_id, DefaultAccount::balance())
            .unwrap(),
        1_000_000_000 - 1_000
    );

    // test snapshot/restore

    let dir = tempdir().unwrap();
    let store = Store::<Blake2b>::new(&dir.path()).unwrap();

    let snapshot = store.persist(&mut network).unwrap();

    // assert that snapshotted version still returns same balance

    assert_eq!(
        network
            .call_contract(genesis_id, DefaultAccount::balance())
            .unwrap(),
        1_000_000_000 - 1_000
    );

    let mut restored = store.restore(&snapshot).unwrap();

    // restored network gives same result

    assert_eq!(
        restored
            .call_contract(genesis_id, DefaultAccount::balance())
            .unwrap(),
        1_000_000_000 - 1_000
    );
}

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
    let schedule = Schedule::default();
    let genesis_builder =
        ContractModule::new(contract_code!("factorial"), &schedule).unwrap();

    let genesis = genesis_builder.build().unwrap();

    // New genesis network with initial value
    let mut network = NetworkState::genesis(genesis, 1_000_000_000).unwrap();

    let genesis_id = *network.genesis_id();

    let n = 6;
    assert_eq!(
        network.call_contract(genesis_id, factorial(n)).unwrap(),
        factorial_reference(n)
    );
}

#[test]
fn factorial_with_limit() {
    use factorial::factorial;

    fn factorial_reference(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            n * factorial_reference(n - 1)
        }
    }
    let schedule = Schedule::default();
    let genesis_builder =
        ContractModule::new(contract_code!("factorial"), &schedule).unwrap();

    let genesis = genesis_builder.build().unwrap();

    // New genesis network with initial value
    let mut network = NetworkState::genesis(genesis, 1_000_000_000).unwrap();

    let genesis_id = *network.genesis_id();
    let mut gas_meter = GasMeter::with_limit(1_000_000_000);
    println!(
        "Before call: gas_meter={:?} (spent={})",
        gas_meter,
        gas_meter.spent()
    );

    let n = 6;
    assert_eq!(
        network
            .call_contract_with_limit(genesis_id, factorial(n), &mut gas_meter)
            .unwrap(),
        factorial_reference(n)
    );

    println!(
        "After call: gas_meter={:?} (spent={})",
        gas_meter,
        gas_meter.spent()
    );
}

#[test]
#[should_panic]
fn panic_propagation() {
    use dusk_abi::ContractCall;

    let schedule = Schedule::default();
    let genesis_builder =
        ContractModule::new(contract_code!("panic"), &schedule).unwrap();

    let genesis = genesis_builder.build().unwrap();

    // New genesis network with initial value
    let mut network = NetworkState::genesis(genesis, 1_000_000_000).unwrap();

    let genesis_id = *network.genesis_id();

    network
        .call_contract::<()>(genesis_id, ContractCall::nil())
        .unwrap();
}
