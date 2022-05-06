// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::CanonError;
use counter::Counter;
use stack::*;
use map::*;
use rusk_vm::{Contract, GasMeter, NetworkState, Schedule};
use microkelvin::{BackendCtor, DiskBackend, Persistence};


fn execute_counter_contract() -> u64 {
    Persistence::with_backend(&testbackend(), |_| Ok(())).unwrap();
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(contract_id, 0, counter::INCREMENT, &mut gas)
        .expect("Transaction error");

    network
        .query::<_, i32>(contract_id, 0, counter::READ_VALUE, &mut gas)
        .expect("Query error");

    gas.spent()
}

fn execute_stack_single_push_pop_contract() -> u64 {
    Persistence::with_backend(&testbackend(), |_| Ok(())).unwrap();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::<u64>::new();

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, Result<(), CanonError>>(contract_id, 0, (stack::PUSH, 100u64), &mut gas).unwrap()
        .expect("Transaction error");

    network
        .query::<_, Result<Option<u64>, CanonError>>(contract_id, 0, (stack::PEEK, 0), &mut gas).unwrap()
        .expect("Query error");

    gas.spent()
}

fn execute_stack_multi_push_pop_contract(count: u64) -> u64 {
    Persistence::with_backend(&testbackend(), |_| Ok(())).unwrap();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::<u64>::new();

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, Result<(), CanonError>>(contract_id, 0, (stack::PUSHMULTI, count), &mut gas).unwrap()
        .expect("Transaction error");

    network
        .transact::<_, Result<Option<u64>, CanonError>>(contract_id, 0, (stack::POPMULTI, count), &mut gas).unwrap()
        .expect("Query error");

    gas.spent()
}

fn testbackend() -> BackendCtor<DiskBackend> {
    BackendCtor::new(DiskBackend::ephemeral)
}

fn execute_multiple_transactions_stack_contract(count: u64) -> u64 {
    Persistence::with_backend(&testbackend(), |_| Ok(())).unwrap();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");
    let stack = Stack::<u64>::new();

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(10_000_000_000);

    for i in 0..count {
        network
            .transact::<_, Result<(), CanonError>>(contract_id, 0, (stack::PUSH, i), &mut gas).unwrap()
            .expect("Transaction error");
    }

    gas.spent()
}

fn execute_multiple_register_contract(count: u64) -> u64 {
    Persistence::with_backend(&testbackend(), |_| Ok(())).unwrap();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/map.wasm");
    let register = Map::new();

    let contract = Contract::new(register, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(10_000_000_000);

    for i in 0..count {
        network
            .transact::<_, Option<u32>>(contract_id, 0, (map::SET, i, i as u32), &mut gas)
            .expect("Transaction error");

        network
            .query::<_, Option<u32>>(contract_id, 0, (map::GET, i, i as u32), &mut gas)
            .expect("Query error");
    }

    gas.spent()
}


#[test]
fn measure_gas_usage() {
    println!("gas usage:");
    println!("counter                                 {}", execute_counter_contract());
    println!("stack single push/pop                   {}", execute_stack_single_push_pop_contract());
    println!("stack multiple push/pop ({})         {}", 65536, execute_stack_multi_push_pop_contract(65536));
    println!("stack multiple transactions push ({}) {}", 8192, execute_multiple_transactions_stack_contract(8192));
    println!("hamt single insert/get                  {}", execute_multiple_register_contract(1));
    println!("hamt multiple insert/get ({})         {}", 8192, execute_multiple_register_contract(8192));
}

// Task:                                   Canon             Rkyv
// counter                                 5493              10732
// stack single push/pop                   31937             15299
// stack multiple push/pop (65536)         806,802,898       823,847,616
// stack multiple transactions push (8192) 1,290,562,425     407,717,312
// hamt single insert/get                  41472             20796
// hamt multiple insert/get (8192)         1,606,679,210     162,128,877


