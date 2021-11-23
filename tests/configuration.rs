// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use rusk_vm::{Contract, GasMeter, NetworkState, Schedule};

fn execute_contract(network: &mut NetworkState) -> u64 {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

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

fn execute_contract_with_file(config_path: &str) -> u64 {
    let mut network =
        NetworkState::with_config_file(Some(config_path.to_string())).unwrap();
    execute_contract(&mut network)
}

fn execute_contract_with_schedule(schedule: &Schedule) -> u64 {
    let mut network = NetworkState::with_schedule(schedule);
    execute_contract(&mut network)
}

#[test]
fn change_gas_cost_per_op_with_configuration_file() {
    assert!(execute_contract_with_file("tests/config/config.toml") < 10_000);
    assert!(
        execute_contract_with_file("tests/config/high_cost_config.toml")
            > 10_000_000
    );
}

#[test]
fn change_gas_cost_per_op_with_schedule() {
    let schedule = Schedule::default();
    assert!(execute_contract_with_schedule(&schedule) < 10_000);
    // let high_cost_schedule = Schedule::default();
    // fill out high_cost_schedule
    // assert!(
    //     execute_contract_with_schedule(&high_cost_schedule)
    //         > 10_000_000
    // );
}

#[test]
fn no_gas_consumption_when_metering_is_off() {
    assert_eq!(
        execute_contract_with_file("tests/config/no_metering_config.toml"),
        0
    );
}

#[test]
fn missing_configuration_file() {
    assert!(matches!(
        NetworkState::with_config_file(Some("missing_config.toml".to_string())),
        Err(rusk_vm::VMError::ConfigurationFileError(_))
    ));
}

#[test]
fn invalid_configuration_file() {
    assert!(matches!(
        NetworkState::with_config_file(Some(
            "tests/config/invalid_config.toml".to_string()
        )),
        Err(rusk_vm::VMError::ConfigurationError(_))
    ));
}
