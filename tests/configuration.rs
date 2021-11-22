// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use rusk_vm::{Contract, GasMeter, NetworkState};

fn execute_contract(config_path: &str) -> u64 {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network =
        NetworkState::with_config(Some(config_path.to_string())).unwrap();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
        .expect("Transaction error");

    network
        .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
        .expect("Query error");

    println!("gas spent={} for {}", gas.spent(), config_path);
    gas.spent()
}

#[test]
fn change_gas_cost_per_op_via_configuration() {
    assert!(execute_contract("tests/config/config.toml") < 10_000);
    assert!(
        execute_contract("tests/config/high_cost_config.toml") > 10_000_000
    );
}

#[test]
fn no_gas_consumption_when_metering_is_off() {
    assert_eq!(execute_contract("tests/config/no_metering_config.toml"), 0);
}

#[test]
fn missing_configuration_file() {
    assert!(matches!(
        NetworkState::with_config(Some("missing_config.toml".to_string())),
        Err(rusk_vm::VMError::ConfigurationFileError(_))
    ));
}

#[test]
fn invalid_configuration_file() {
    assert!(matches!(
        NetworkState::with_config(Some(
            "tests/config/invalid_config.toml".to_string()
        )),
        Err(rusk_vm::VMError::ConfigurationError(_))
    ));
}
