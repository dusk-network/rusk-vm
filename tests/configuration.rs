// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use rusk_vm::{Contract, GasMeter, NetworkState, Schedule};
use std::collections::HashMap;

async fn execute_contract(network: &mut NetworkState) -> u64 {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let contract_id = network.deploy(contract).await.expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(contract_id, 0, counter::INCREMENT, &mut gas)
        .await
        .expect("Transaction error");

    network
        .query::<_, i32>(contract_id, 0, counter::READ_VALUE, &mut gas)
        .await
        .expect("Query error");

    gas.spent()
}

async fn execute_contract_with_schedule(schedule: &Schedule) -> u64 {
    let mut network = NetworkState::with_schedule(schedule);
    execute_contract(&mut network).await
}

#[tokio::test]
async fn change_gas_cost_per_op_with_schedule() {
    let schedule = Schedule::default();
    assert!(execute_contract_with_schedule(&schedule).await < 10_000);

    let per_type_op_cost: HashMap<String, u32> = [
        ("bit", 10000),
        ("add", 10000),
        ("mul", 10000),
        ("div", 10000),
        ("load", 10000),
        ("store", 10000),
        ("const", 10000),
        ("local", 10000),
        ("global", 10000),
        ("flow", 10000),
        ("integer_comp", 10000),
        ("float_comp", 10000),
        ("float", 10000),
        ("conversion", 10000),
        ("float_conversion", 10000),
        ("reinterpret", 10000),
        ("unreachable", 10000),
        ("nop", 10000),
        ("current_mem", 10000),
        ("grow_mem", 10000),
    ]
    .iter()
    .cloned()
    .map(|(s, c)| (s.to_string(), c))
    .collect();

    let high_cost_schedule = Schedule {
        per_type_op_cost,
        ..Schedule::with_version(1)
    };
    assert!(
        execute_contract_with_schedule(&high_cost_schedule).await > 10_000_000
    );
}

#[tokio::test]
async fn no_gas_consumption_when_metering_is_off() {
    let no_metering_schedule = Schedule {
        has_metering: false,
        ..Schedule::with_version(2)
    };
    assert_eq!(
        execute_contract_with_schedule(&no_metering_schedule).await,
        0
    );
}
