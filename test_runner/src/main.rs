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

use counter::*;
use stack::*;
// use map::*;
use microkelvin::*;
use rusk_vm::*;
use crate::rusk_uplink::StoreContext;

static mut PATH: String = String::new();

const STACK_TEST_SIZE: u64 = 5000;
const CONFIRM_STACK_METHOD: u32 = 2;

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
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/counter.wasm"
    );

    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store.clone());

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

    let persist_id = network.persist(store).expect("Error in persistence");

    let file_path = PathBuf::from(unsafe { &PATH }).join("counter_persist_id");

    persist_id.write(file_path)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("counter_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn initialize_stack(
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    let stack = Stack::new();

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/stack.wasm"
    );

    let contract = Contract::new(&stack, code.to_vec(), &store);

    let mut network = NetworkState::new(store.clone());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = STACK_TEST_SIZE;

    for i in 0..N {
        if i % 100 == 0 {
            println!("push ===> {}", i);
        }
        network
            .transact(contract_id, 0, Push::new(i), &mut gas)
            .unwrap();
    }

    network.commit();

    println!("before stack persist");
    let persist_id = network.persist(store).expect("Error in persistence");
    println!("after stack persist");

    let file_path = PathBuf::from(unsafe { &PATH }).join("stack_persist_id");

    persist_id.write(file_path)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("stack_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn initialize_stack_multi(
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    let stack = Stack::new();

    let code = include_bytes!(
        "../../target/wasm32-unknown-unknown/release/stack.wasm"
    );

    let contract = Contract::new(&stack, code.to_vec(), &store);

    let mut network = NetworkState::new(store.clone());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(100_000_000_000);

    const N: u64 = STACK_TEST_SIZE;

    println!("pushmulti ===> {}", N);
    network
        .transact(contract_id, 0, PushMulti::new(N), &mut gas)
        .unwrap();

    println!("before network commit");
    network.commit();
    println!("after network commit");

    println!("before stack persist");
    let persist_id = network.persist(store).expect("Error in persistence");
    println!("after stack persist");

    let file_path = PathBuf::from(unsafe { &PATH }).join("stack_persist_id");

    persist_id.write(file_path)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("stack_contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

// fn initialize_map(
//     backend: &BackendCtor<DiskBackend>,
// ) -> Result<(), Box<dyn Error>> {
//     let counter = Map::new();
//
//     let code =
//         include_bytes!("../../target/wasm32-unknown-unknown/release/map.wasm");
//
//     let contract = Contract::new(counter, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     for i in 0..MAP_SIZE {
//         network
//             .transact::<_, Option<u32>>(
//                 contract_id,
//                 0,
//                 (map::SET, i, i as u32),
//                 &mut gas,
//             )
//             .unwrap();
//     }
//
//     network.commit();
//
//     let persist_id = network.persist(backend).expect("Error in persistence");
//
//     let file_path = PathBuf::from(unsafe { &PATH }).join("map_persist_id");
//
//     persist_id.write(file_path)?;
//
//     let contract_id_path =
//         PathBuf::from(unsafe { &PATH }).join("map_contract_id");
//
//     fs::write(&contract_id_path, contract_id.as_bytes())?;
//
//     Ok(())
// }

fn confirm_counter(
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    println!("confirm");
    let file_path = PathBuf::from(unsafe { &PATH }).join("counter_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new(store.clone())
        .restore(store, state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("counter_contract_id");
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

// fn confirm_map(
//     _backend: &BackendCtor<DiskBackend>,
// ) -> Result<(), Box<dyn Error>> {
//     let file_path = PathBuf::from(unsafe { &PATH }).join("map_persist_id");
//     let state_id = NetworkStateId::read(file_path)?;
//
//     let mut network = NetworkState::new()
//         .restore(state_id)
//         .map_err(|_| PersistE)?;
//
//     let contract_id_path =
//         PathBuf::from(unsafe { &PATH }).join("map_contract_id");
//     let buf = fs::read(&contract_id_path)?;
//
//     let contract_id = ContractId::from(buf);
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     for i in 0..MAP_SIZE {
//         assert_eq!(
//             network
//                 .query::<_, Option<u32>>(
//                     contract_id,
//                     0,
//                     (map::GET, i),
//                     &mut gas
//                 )
//                 .unwrap(),
//             Some(i as u32)
//         );
//     }
//
//     Ok(())
// }

fn remove_disk_store(path: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
    let _ = Command::new("rm")
        .arg(PathBuf::from(path.as_ref()).join("storage").to_str().expect("Path join works"))
        .output()?;
    Ok(())
}

fn create_disk_store(path: impl AsRef<str>) -> Result<StoreContext, Box<dyn Error>> {
    let _output = Command::new("mkdir")
        .arg(path.as_ref())
        .output()
        .expect("failed to execute process");

    let store =
        StoreRef::new(HostStore::with_file(path.as_ref())?);
    Ok(store)
}

fn move_stack_elements_to_memory(stack: &mut Stack) {
    /*
    note - brute force method to bring all leaves to memory
     */
    // let mut temp_store: Vec<u64> = Vec::new();
    // for i in 0..N {
    //     temp_store.push(stack.pop().unwrap());
    // }
    // temp_store.reverse();
    // for i in temp_store {
    //     stack.push(i);
    // }

    /*
    note - simpler method to bring all leaves to memory
     */
    let branch_mut = stack.inner.walk_mut(All).expect("Some(Branch)");
    for leaf in branch_mut {
        *leaf += 0;
    }
}

fn confirm_stack1(
    store_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    println!("confirm");
    let store1 = StoreRef::new(HostStore::with_file(store_path.as_ref())?);
    let file_path = PathBuf::from(unsafe { &PATH }).join("stack_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new(store1.clone())
        .restore(store1.clone(), state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("stack_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    const N: u64 = STACK_TEST_SIZE;

    /*
    we need to deserialize contract state so that it is fully in memory for store1
    then we need to serialize it to store2
     */
    let mut stack_state = network.deserialize_from_contract_state::<Stack>(store1, contract_id)?;
    for i in 0..N {
        assert_eq!(Some(i), stack_state.peek(i));
    }

    // return Ok(()); // return here for big storage footprint

    remove_disk_store(store_path.as_ref())?;
    let store2 = create_disk_store(store_path.as_ref())?;

    /*
    enforce moving of the entire state to memory
    there should be an easier and more performant way to do it
     */
    for i in 0..N {
        let ii = stack_state.peek(i);
        println!("peek after store disk removed - of {} = {:?}", i, ii);
    }

    /*
    now we should have all data in stack_state in memory
     */
    move_stack_elements_to_memory(&mut stack_state);

    /*
    serialize the state and put it into the contract
     */
    network.serialize_into_contract_state(store2.clone(), contract_id, &stack_state)?;
    network.commit();
    /*
    now we can persist everything
     */
    store2.persist().expect("Error in persistence");
    let persist_id2 = network.persist(store2.clone()).expect("Error in persistence");

    /*
    we can now restore and make sure that the state has been preserved
     */
    let mut network = NetworkState::new(store2.clone())
        .restore(store2.clone(), persist_id2)
        .map_err(|_| PersistE)?;

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..N {
        let ii = network
            .transact(contract_id, 0, Pop::new(), &mut gas)
            .unwrap();
        if (N > 1000) && (i % 100 == 0) {
            println!("checking pop ===> {} {:?}", N - 1 - i, ii);
        }
        assert_eq!(Some(N-1-i), ii);
    }
    /*
    ok - state has been preserved using much less storage as the entire history is now gone
     */

    Ok(())
}

fn confirm_stack2(
    store_path: impl AsRef<str>,
) -> Result<(), Box<dyn Error>> {
    println!("confirm");
    let store1 = StoreRef::new(HostStore::with_file(store_path.as_ref())?);
    let store2 = StoreRef::new(HostStore::with_file("/tmp/rusk-vm-test-runner-temp-dir2")?);
    let file_path = PathBuf::from(unsafe { &PATH }).join("stack_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::with_target_store(store1.clone(), store2.clone())
        .restore(store1.clone(), state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("stack_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    const N: u64 = STACK_TEST_SIZE;

    let mut gas = GasMeter::with_limit(100_000_000_000);
    network
        .transact(contract_id, 0, StoreState::new(), &mut gas)
        .unwrap();

    /*
    now we can persist everything
     */
    network.commit();
    let persist_id2 = network.persist(store2.clone()).expect("Error in persistence");

    /*
    we can now restore and make sure that the state has been preserved
     */
    remove_disk_store(store_path.as_ref())?; // to make sure we don't access old 'big' store

    let mut network = NetworkState::new(store2.clone())
        .restore(store2.clone(), persist_id2)
        .map_err(|_| PersistE)?;

    let mut gas = GasMeter::with_limit(100_000_000_000);
    for i in 0..N {
        let ii = network
            .transact(contract_id, 0, Pop::new(), &mut gas)
            .unwrap();
        if (N > 1000) && (i % 100 == 0) {
            println!("checking pop ===> {} {:?}", N - 1 - i, ii);
        }
        assert_eq!(Some(N-1-i), ii);
    }
    /*
    ok - state has been preserved using much less storage as the entire history is now gone
     */

    Ok(())
}

fn confirm_stack_multi(
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    println!("confirm");
    let file_path = PathBuf::from(unsafe { &PATH }).join("stack_persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new(store.clone())
        .restore(store, state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path =
        PathBuf::from(unsafe { &PATH }).join("stack_contract_id");
    let buf = fs::read(&contract_id_path)?;

    let contract_id = ContractId::from(buf);

    let mut gas = GasMeter::with_limit(100_000_000_000);
    //
    const N: u64 = STACK_TEST_SIZE;

    let mut expected_sum = 0u64;
    for i in 0..N {
        expected_sum += i;
    }

    println!("popmulti ===> {}", N);
    let sum = network
        .transact(contract_id, 0, PopMulti::new(N), &mut gas)
        .unwrap();
    assert_eq!(sum, expected_sum);

    Ok(())
}

fn initialize(
    store: StoreContext,
) -> Result<(), Box<dyn Error>> {
    // initialize_counter(store.clone())?;
    initialize_stack(store.clone())?;
    // initialize_stack_multi(store)?;
    Ok(())
}

fn confirm(store: StoreContext, path: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
    // confirm_counter(store.clone())?;
    if CONFIRM_STACK_METHOD == 2 {
        confirm_stack2(path)?;
    } else {
        confirm_stack1(path)?;
    }
    // confirm_stack_multi(store)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let store = unsafe {
        PATH = args[1].clone();
        // PATH = String::from("/tmp/rusk-vm-test-runner-temp-dir");
        StoreRef::new(HostStore::with_file(&PATH)?)
    };

    // initialize(store.clone())?;
    // confirm(store.clone())

    match &*args[2] {
        "initialize" => initialize(store),
        "confirm" => confirm(store, unsafe { &PATH } ),
        "test_both" => {
            initialize(store.clone())?;
            confirm(store, unsafe { &PATH })
        }
        _ => Err(Box::new(IllegalArg)),
    }
}