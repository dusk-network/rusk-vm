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
use std::process::Command;

use crate::rusk_uplink::StoreContext;
use byteorder::{LittleEndian, WriteBytesExt};
use counter::*;
use microkelvin::*;
use register::*;
use rusk_vm::*;
use stack::*;

const STACK_TEST_SIZE: u64 = 5000;
const REGISTER_TEST_SIZE: u64 = 5000;
const STACK_REGISTER_TEST_SIZE: u64 = 5000;

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

fn initialize_counter(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut backend = Backend::new(source_path.as_ref())?;
    let network = &mut *backend;

    let counter = Counter::new(99);

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/counter.wasm"
    );

    let contract = Contract::new(&counter, code.to_vec(), &network.get_store_ref());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        99
    );

    network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .unwrap();

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        100
    );

    network.commit();

    backend.persist()?;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("counter_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    println!("end of initialize counter {}", source_path.as_ref());

    Ok(())
}

fn initialize_stack(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let stack = Stack::new();

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/stack.wasm"
    );

    let mut backend = Backend::new(source_path.as_ref())?;
    let network = &mut *backend;

    let contract = Contract::new(&stack, code.to_vec(), &network.get_store_ref());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = STACK_TEST_SIZE;

    for i in 0..N {
        if (N > 1000) && (i % 100 == 0) {
            println!("push ===> {}", i);
        }
        network
            .transact(contract_id, 0, Push::new(i), &mut gas)
            .unwrap();
    }

    network.commit();

    println!("initialize stack - persist to disk");
    backend.persist()?;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn secret_data_from_int(secret_data: &mut [u8; 32], i: u64) {
    secret_data
        .as_mut_slice()
        .write_u64::<LittleEndian>(i)
        .expect("Unable to write");
}

fn initialize_register(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut backend = Backend::new(source_path.as_ref())?;
    let network = &mut *backend;
    let stack = Stack::new();

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/register.wasm"
    );

    let contract = Contract::new(&stack, code.to_vec(), &network.get_store_ref());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = REGISTER_TEST_SIZE;

    for i in 0..N {
        if (N > 1000) && (i % 100 == 0) {
            println!("gossip ===> {}", i);
        }
        let mut secret_data: [u8; 32] = [0u8; 32];
        secret_data_from_int(&mut secret_data, i);
        let secret_hash = SecretHash::new(secret_data);

        network
            .transact(contract_id, 0, Gossip::new(secret_hash), &mut gas)
            .unwrap();
    }

    network.commit();

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("register_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..N {
        let mut secret_data: [u8; 32] = [0u8; 32];
        secret_data_from_int(&mut secret_data, i);
        let secret_hash = SecretHash::new(secret_data);
        let ii = network
            .query(contract_id, 0, NumSecrets::new(secret_hash), &mut gas)
            .unwrap();
        if (N > 1000) && (i % 100 == 0) {
            println!("confirming num secrets for {} ===> {} ", i, ii);
        }
    }

    backend.persist()?;

    Ok(())
}

fn initialize_stack_and_register(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut backend = Backend::new(source_path.as_ref())?;
    let network = &mut *backend;
    let stack = Stack::new();

    let code_stack = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/stack.wasm"
    );
    let code_register = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/register.wasm"
    );

    let contract_stack = Contract::new(&stack, code_stack.to_vec(), &network.get_store_ref());
    let contract_register =
        Contract::new(&stack, code_register.to_vec(), &network.get_store_ref());

    let contract_id_stack = network.deploy(contract_stack).unwrap();
    let contract_id_register = network.deploy(contract_register).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = STACK_REGISTER_TEST_SIZE;

    for i in 0..N {
        if (N > 1000) && (i % 100 == 0) {
            println!("push to NStack and insert into Hamt ===> {}", i);
        }
        network
            .transact(contract_id_stack, 0, Push::new(i), &mut gas)
            .unwrap();
        let mut secret_data: [u8; 32] = [0u8; 32];
        secret_data_from_int(&mut secret_data, i);
        let secret_hash = SecretHash::new(secret_data);
        network
            .transact(
                contract_id_register,
                0,
                Gossip::new(secret_hash),
                &mut gas,
            )
            .unwrap();
    }

    network.commit();

    backend.persist()?;

    let contract_id_stack_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");

    fs::write(&contract_id_stack_path, contract_id_stack.as_bytes())?;

    let contract_id_register_path =
        PathBuf::from(source_path.as_ref()).join("register_contract_id");

    fs::write(&contract_id_register_path, contract_id_register.as_bytes())?;

    Ok(())
}

fn initialize_stack_multi(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut backend = Backend::new(source_path.as_ref())?;
    let network = &mut *backend;
    let stack = Stack::new();

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/stack.wasm"
    );

    let contract = Contract::new(&stack, code.to_vec(), &network.get_store_ref());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = STACK_TEST_SIZE;

    network
        .transact(contract_id, 0, PushMulti::new(N), &mut gas)
        .unwrap();

    network.commit();

    backend.persist()?;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn confirm_counter(
    source_path: impl AsRef<str>,
    target_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut gas = GasMeter::with_limit(100_000_000_000);
    Backend::compact(
        PathBuf::from(source_path.as_ref()),
        PathBuf::from(target_path.as_ref()),
        &mut gas,
    )?;
    let mut backend = Backend::restore(target_path.as_ref())?;
    let network = &mut *backend;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("counter_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        100
    );

    Ok(())
}

fn confirm_stack(
    source_path: impl AsRef<str>,
    target_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut gas = GasMeter::with_limit(100_000_000_000);
    println!("confirm stack 3 - consolidate to disk");

    NetworkState::compact(source_path.as_ref(), target_path.as_ref(), &mut gas)?;

    let mut backend = Backend::restore(target_path.as_ref())?;
    let network = &mut *backend;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..STACK_TEST_SIZE {
        let ii = network
            .transact(contract_id, 0, Pop::new(), &mut gas)
            .unwrap();
        if (STACK_TEST_SIZE > 1000) && (i % 100 == 0) {
            println!("checking pop ===> {} {:?}", STACK_TEST_SIZE - 1 - i, ii);
        }
        assert_eq!(Some(STACK_TEST_SIZE - 1 - i), ii);
    }
    /*
    ok - state has been preserved using much less storage
     */

    Ok(())
}

fn confirm_register(
    source_path: impl AsRef<str>,
    target_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut gas = GasMeter::with_limit(100_000_000_000);
    NetworkState::compact(source_path.as_ref(), target_path.as_ref(), &mut gas)?;
    let mut backend = Backend::restore(target_path.as_ref())?;
    let network = &mut *backend;

    let contract_id_register_path =
        PathBuf::from(source_path.as_ref()).join("register_contract_id");
    let buf = fs::read(&contract_id_register_path)?;
    let register_contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..REGISTER_TEST_SIZE {
        let mut secret_data: [u8; 32] = [0u8; 32];
        secret_data_from_int(&mut secret_data, i);
        let secret_hash = SecretHash::new(secret_data);
        let ii = network
            .query(
                register_contract_id,
                0,
                NumSecrets::new(secret_hash),
                &mut gas,
            )
            .unwrap();
        if (REGISTER_TEST_SIZE > 1000) && (i % 100 == 0) {
            println!("checking num secrets (Hamt) ===> {} {} ", i, ii);
        }
    }
    /*
    ok - state has been preserved using much less storage
     */
    Ok(())
}

fn confirm_stack_and_register(
    source_path: impl AsRef<str>,
    target_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let mut gas = GasMeter::with_limit(100_000_000_000);
    NetworkState::compact(source_path.as_ref(), target_path.as_ref(), &mut gas)?;
    let mut backend = Backend::restore(target_path.as_ref())?;
    let network = &mut *backend;

    /*
       confirm stack
    */

    let contract_id_stack_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");
    let buf = fs::read(&contract_id_stack_path)?;
    let stack_contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..STACK_REGISTER_TEST_SIZE {
        let ii = network
            .transact(stack_contract_id, 0, Pop::new(), &mut gas)
            .unwrap();
        if (STACK_REGISTER_TEST_SIZE > 1000) && (ii.unwrap() % 100 == 0) {
            println!(
                "checking pop (NStack) ===> {} {:?}",
                STACK_REGISTER_TEST_SIZE - 1 - i,
                ii
            );
        }
        assert_eq!(Some(STACK_REGISTER_TEST_SIZE - 1 - i), ii);
    }

    /*
       confirm register
    */
    let contract_id_register_path =
        PathBuf::from(source_path.as_ref()).join("register_contract_id");
    let buf = fs::read(&contract_id_register_path)?;
    let register_contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..STACK_REGISTER_TEST_SIZE {
        let mut secret_data: [u8; 32] = [0u8; 32];
        secret_data_from_int(&mut secret_data, i);
        let secret_hash = SecretHash::new(secret_data);
        let ii = network
            .query(
                register_contract_id,
                0,
                NumSecrets::new(secret_hash),
                &mut gas,
            )
            .unwrap();
        if (STACK_REGISTER_TEST_SIZE > 1000) && (i % 100 == 0) {
            println!("checking num secrets (Hamt) ===> {} {} ", i, ii);
        }
    }

    Ok(())
}

fn confirm_stack_multi(
    source_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    let store = StoreRef::new(HostStore::with_file(source_path.as_ref())?);
    let file_path = PathBuf::from(source_path.as_ref()).join("persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new(store.clone())
        .restore_from_store(store, state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(source_path.as_ref()).join("stack_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    //
    const N: u64 = STACK_TEST_SIZE;

    let mut expected_sum = 0u64;
    for i in 0..N {
        expected_sum += i;
    }

    let sum = network
        .transact(contract_id, 0, PopMulti::new(N), &mut gas)
        .unwrap();
    assert_eq!(sum, expected_sum);

    Ok(())
}

fn initialize(source_path: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
    // initialize_counter(source_path.as_ref())?;
    // initialize_stack(source_path.as_ref())?;
    // initialize_register(source_path.as_ref())?;
    initialize_stack_and_register(source_path.as_ref())?;
    // initialize_stack_multi(source_path.as_ref())?;
    Ok(())
}

fn confirm(
    source_path: impl AsRef<str>,
    target_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    // confirm_counter(source_path.as_ref(), target_path.as_ref())?;
    // confirm_stack(source_path.as_ref(), target_path.as_ref())?;
    // confirm_register(source_path.as_ref(), target_path.as_ref())?;
    confirm_stack_and_register(source_path.as_ref(), target_path.as_ref())?;
    // confirm_stack_multi(source_path.as_ref())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let source_path = args[2].clone();

    match &*args[1] {
        "initialize" => initialize(source_path),
        "confirm" => {
            let target_path = args[3].clone();
            confirm(source_path, target_path)
        }
        _ => Err(Box::new(IllegalArg)),
    }
}
