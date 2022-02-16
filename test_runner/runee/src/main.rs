use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use counter::*;
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

fn initialize(
    backend: &BackendCtor<DiskBackend>,
) -> Result<(), Box<dyn Error>> {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/counter.wasm"
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

    let file_path = PathBuf::from(unsafe { &PATH }).join("persist_id");

    persist_id.write(file_path)?;

    let contract_id_path = PathBuf::from(unsafe { &PATH }).join("contract_id");

    fs::write(&contract_id_path, contract_id.as_bytes())?;

    Ok(())
}

fn confirm(_backend: &BackendCtor<DiskBackend>) -> Result<(), Box<dyn Error>> {
    let file_path = PathBuf::from(unsafe { &PATH }).join("persist_id");
    let state_id = NetworkStateId::read(file_path)?;

    let mut network = NetworkState::new()
        .restore(state_id)
        .map_err(|_| PersistE)?;

    let contract_id_path = PathBuf::from(unsafe { &PATH }).join("contract_id");
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

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    println!("path {:?}", args[1]);

    let backend = unsafe {
        PATH = args[1].clone();
        BackendCtor::new(|| DiskBackend::new(&PATH))
    };

    Persistence::with_backend(&backend, |_| Ok(()));

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
