// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use byteorder::{LittleEndian, WriteBytesExt};
use counter::Counter;
use microkelvin::{HostStore, StoreRef};
use register::*;
use rusk_vm::{Contract, GasMeter, NetworkState, Schedule};
use stack::*;

fn execute_counter_contract() -> u64 {
    let schedule = Schedule::default();
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);
    let mut network = NetworkState::with_schedule(store, &schedule);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .expect("Transaction error");

    network
        .query(contract_id, 0, counter::ReadValue, &mut gas)
        .expect("Query error");

    gas.spent()
}

fn execute_stack_single_push_pop_contract() -> u64 {
    let schedule = Schedule::default();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::new();

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&stack, code.to_vec(), &store);
    let mut network = NetworkState::with_schedule(store, &schedule);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, stack::Push::new(100), &mut gas)
        .expect("Transaction error");

    network
        .query(contract_id, 0, stack::Peek::new(0), &mut gas)
        .expect("Query error");

    gas.spent()
}

fn execute_stack_multi_push_pop_contract(count: u64) -> u64 {
    let schedule = Schedule::default();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::new();

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&stack, code.to_vec(), &store);
    let mut network = NetworkState::with_schedule(store, &schedule);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, stack::PushMulti::new(count), &mut gas)
        .expect("Transaction error");

    network
        .transact(contract_id, 0, stack::PopMulti::new(count), &mut gas)
        .expect("Query error");

    gas.spent()
}

fn execute_multiple_transactions_stack_contract(count: u64) -> u64 {
    let schedule = Schedule::default();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::new();

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&stack, code.to_vec(), &store);
    let mut network = NetworkState::with_schedule(store, &schedule);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..count {
        network
            .transact(contract_id, 0, Push::new(i), &mut gas)
            .unwrap();
    }

    gas.spent()
}

fn execute_multiple_register_contract(count: u64) -> u64 {
    let schedule = Schedule::default();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/register.wasm"
    );
    let register = Register::new();

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&register, code.to_vec(), &store);
    let mut network = NetworkState::with_schedule(store, &schedule);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    fn secret_data_from_int(secret_data: &mut [u8; 32], i: u64) {
        secret_data
            .as_mut_slice()
            .write_u64::<LittleEndian>(i)
            .expect("Unable to write");
    }

    let mut secret_data: [u8; 32] = [0u8; 32];
    secret_data_from_int(&mut secret_data, 5);
    let secret_hash = SecretHash::new(secret_data);

    for _ in 0..count {
        network
            .transact(contract_id, 0, Gossip::new(secret_hash), &mut gas)
            .expect("Transaction error");

        network
            .query(contract_id, 0, NumSecrets::new(secret_hash), &mut gas)
            .expect("Query error");
    }

    gas.spent()
}

#[test]
fn measure_gas_usage() {
    println!("gas usage:");
    println!(
        "counter                                 {}",
        execute_counter_contract()
    );
    println!(
        "stack single push/pop                   {}",
        execute_stack_single_push_pop_contract()
    );
    println!(
        "stack multiple push/pop ({})         {}",
        65536,
        execute_stack_multi_push_pop_contract(65536)
    );
    println!(
        "stack multiple transactions push ({}) {}",
        8192,
        execute_multiple_transactions_stack_contract(8192)
    );
    println!(
        "hamt single insert/get                  {}",
        execute_multiple_register_contract(1)
    );
    println!(
        "hamt multiple insert/get ({})         {}",
        8192,
        execute_multiple_register_contract(8192)
    );
}
