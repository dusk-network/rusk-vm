// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

use counter::*;
use map::*;
use microkelvin::*;
use rusk_vm::*;

static mut PATH: String = String::new();

#[derive(Debug)]
struct IllegalArg;
impl Error for IllegalArg {}

impl Display for IllegalArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Illegal arg")
    }
}

#[derive(Debug)]
struct PersistE;
impl Error for PersistE {}

impl Display for PersistE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Persist Error")
    }
}

const MAP_SIZE: u8 = 64;

fn initialize_counter(
    backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/counter.wasm"
    );

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query::<_, i32>(contract_id, 0, counter::READ_VALUE, &mut gas)
            .unwrap(),
        99
    );

    network
        .transact::<_, ()>(contract_id, 0, counter::INCREMENT, &mut gas)
        .unwrap();

    assert_eq!(
        network
            .query::<_, i32>(contract_id, 0, counter::READ_VALUE, &mut gas)
            .unwrap(),
        100
    );

    network.commit();

    let persist_id = network.persist(backend).expect("Error in persistence");

    let file_path = PathBuf::from(unsafe { &PATH }).join("counter_persist_id");

    persist_id.write(file_path)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("counter_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn initialize_map(
    backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    let counter = Map::new();

    let code =
        include_bytes!("../../target/wasm32-unknown-unknown/release/map.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..MAP_SIZE {
        network
            .transact::<_, Option<u32>>(
                contract_id,
                0,
                (map::SET, i, i as u32),
                &mut gas,
            )
            .unwrap();
    }

    network.commit();

    let persist_id = network.persist(backend).expect("Error in persistence");

    let file_path = PathBuf::from(unsafe { &PATH }).join("map_persist_id");

    persist_id.write(file_path)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("map_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn initialize(
    backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    initialize_counter(backend)?;
    initialize_map(backend)?;
    Ok(())
}

fn confirm_counter(
    _backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    let file_path = PathBuf::from(unsafe { &PATH }).join("counter_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new()
        .restore(state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("counter_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query::<_, i32>(contract_id, 0, counter::READ_VALUE, &mut gas)
            .unwrap(),
        100
    );

    Ok(())
}

fn confirm_map(
    _backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    let file_path = PathBuf::from(unsafe { &PATH }).join("map_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new()
        .restore(state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("map_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..MAP_SIZE {
        assert_eq!(
            network
                .query::<_, Option<u32>>(
                    contract_id,
                    0,
                    (map::GET, i),
                    &mut gas
                )
                .unwrap(),
            Some(i as u32)
        );
    }

    Ok(())
}

fn confirm(_backend: &BackendCtor<DiskBackend>) -> Result<(), Box<dyn Error>> {
    confirm_counter(_backend)?;
    confirm_map(_backend)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let backend = unsafe {
        PATH = args[1].clone();
        BackendCtor::new(|| DiskBackend::new(&PATH))
    };

    Persistence::with_backend(&backend, |_| Ok(())).unwrap();

    match &*args[2] {
        "initialize" => initialize(&backend),
        "confirm" => confirm(&backend),
        "test_both" => {
            initialize(&backend)?;
            confirm(&backend)
        }
        _ => Err(Box::new(IllegalArg)),
    }
}
