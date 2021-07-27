use std::error::Error;
use std::fs::File;
use std::io::Read;

use canonical::Canon;
use minimal::Minimal;
use rusk_vm::{Contract, GasMeter, NetworkState};

use nstack::NStack;

fn report_gas<S, A, R>(
    name: &'static str,
    state: S,
    arg: A,
) -> Result<(), Box<dyn Error>>
where
    A: Canon,
    R: Canon,
    S: Canon,
{
    let path =
        format!("../../target/wasm32-unknown-unknown/release/{}.wasm", name);

    let mut file = File::open(path)?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let contract = Contract::new(state, code.to_vec());
    let mut network = NetworkState::default();
    let contract_id = network.deploy(contract).unwrap();
    let limit = 1_000_000_000;
    let mut gas = GasMeter::with_limit(limit);
    network.query::<A, R>(contract_id, arg, &mut gas).unwrap();

    println!(
        "{:?}: wasm size: {:?} gas: {:?}",
        name,
        code.len(),
        gas.spent()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    report_gas::<_, _, ()>("minimal", Minimal, 32)?;
    report_gas::<NStack<u32, ()>, _, ()>(
        "minimal_nstack",
        Default::default(),
        32,
    )?;
    Ok(())
}
