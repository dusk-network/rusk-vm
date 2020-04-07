mod contracts;
mod helpers;

use kelvin::Blake2b;
use std::fs;

use dusk_abi::{
    ContractCall, FeeCall, Provisioners, Signature, StakingCall, TransferCall,
};
use phoenix::PublicKey as PhoenixPK;
use phoenix_abi::{Note, Nullifier, Proof, PublicKey};
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
    let code = contract_code!("transfer");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // Generate some items
    let nullifiers = [Nullifier::default(); Nullifier::MAX];
    let notes = [Note::default(); Note::MAX];
    let proof = Proof::default();

    let succeeded: bool = network
        .call_contract(
            &contract_id,
            ContractCall::new(TransferCall::Transfer {
                nullifiers,
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
    if (succeeded) {
        // Ensure data was written
        fs::metadata("/tmp/rusk-vm-demo/data").unwrap();

        // Clean up
        fs::remove_dir_all("/tmp/rusk-vm-demo").unwrap();
    }
}

#[test]
fn fee() {
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

#[test]
fn staking() {
    let code = contract_code!("transfer");

    let schedule = Schedule::default();
    let contract = Contract::new(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let transfer_id = network.deploy(contract).unwrap();

    let code = contract_code!("staking");
    let contract = Contract::new(code, &schedule).unwrap();
    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // Setup
    let pk = PublicKey::default();

    let call: ContractCall<()> = ContractCall::new(StakingCall::Init {
        address: transfer_id,
        pk: pk,
    })
    .unwrap();

    network.call_contract(&contract_id, call, &mut gas).unwrap();

    // Add provisioner
    let nullifiers = [Nullifier::default(); Nullifier::MAX];
    let notes = [Note::default(); Note::MAX];
    let prov_pk = PhoenixPK::default();

    let call: ContractCall<bool> = ContractCall::new(StakingCall::Stake {
        nullifiers,
        notes,
        pk: prov_pk.into(),
        pk_bls: [2u8; 32],
        expiration: 100,
        value: 1000,
        current_height: 1,
    })
    .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network.call_contract(&contract_id, call, &mut gas).unwrap();

    // Check if he was added properly
    let call: ContractCall<(u64, [u8; 32], u64, u64)> =
        ContractCall::new(StakingCall::GetStake { pk: prov_pk.into() })
            .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);
    let results = network.call_contract(&contract_id, call, &mut gas).unwrap();
    assert_eq!(results.0, 1000);
    assert_eq!(results.1, [2u8; 32]);
    assert_eq!(results.2, 1);
    assert_eq!(results.3, 100);

    // Withdraw the stake
    let call: ContractCall<bool> = ContractCall::new(StakingCall::Withdraw {
        pk: prov_pk.into(),
        // sig: Signature([0u8; 64]),
        current_height: 200,
    })
    .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);
    network.call_contract(&contract_id, call, &mut gas).unwrap();

    // provisioner should have been removed now
    let call: ContractCall<(u64, [u8; 32], u64, u64)> =
        ContractCall::new(StakingCall::GetStake { pk: prov_pk.into() })
            .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);
    let results = network.call_contract(&contract_id, call, &mut gas).unwrap();
    assert_eq!(results.0, 0);
    assert_eq!(results.1, [0u8; 32]);
    assert_eq!(results.2, 0);
    assert_eq!(results.3, 0);

    // Now add a provisioner, and slash them
    let call: ContractCall<bool> = ContractCall::new(StakingCall::Stake {
        nullifiers,
        notes,
        pk: prov_pk.into(),
        pk_bls: [2u8; 32],
        expiration: 100,
        value: 1000,
        current_height: 1,
    })
    .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network.call_contract(&contract_id, call, &mut gas).unwrap();

    let call: ContractCall<bool> = ContractCall::new(StakingCall::Slash {
        pk: prov_pk.into(),
        height: 10,
        step: 2,
        sig1: Signature::from_slice(&[0u8; 64]),
        sig2: Signature::from_slice(&[1u8; 64]),
        msg1: [2u8; 32],
        msg2: [3u8; 32],
    })
    .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network.call_contract(&contract_id, call, &mut gas).unwrap();

    // provisioner should have been removed now
    let call: ContractCall<(u64, [u8; 32], u64, u64)> =
        ContractCall::new(StakingCall::GetStake { pk: prov_pk.into() })
            .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);
    let results = network.call_contract(&contract_id, call, &mut gas).unwrap();
    assert_eq!(results.0, 0);
    assert_eq!(results.1, [0u8; 32]);
    assert_eq!(results.2, 0);
    assert_eq!(results.3, 0);
}
