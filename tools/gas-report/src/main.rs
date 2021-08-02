// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::error::Error;
use std::fs::File;
use std::io::Read;

use canonical::Canon;
use minimal::Minimal;
use minimal_poseidon::Leaf;
use minimal_transfer::MockTransfer;
use rusk_vm::{Contract, GasMeter, NetworkState};

use dusk_poseidon::tree::{PoseidonAnnotation, PoseidonTree};
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

    let encoded_len = state.encoded_len();

    let contract = Contract::new(state, code.to_vec());
    let mut network = NetworkState::default();
    let contract_id = network.deploy(contract).unwrap();
    let limit = 1_000_000_000;
    let mut gas = GasMeter::with_limit(limit);
    network.query::<A, R>(contract_id, arg, &mut gas).unwrap();

    println!(
        "{0: <20} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
        name,
        code.len(),
        core::mem::size_of::<S>(),
        encoded_len,
        gas.spent()
    );
    Ok(())
}

type Tree = PoseidonTree<Leaf, PoseidonAnnotation, 17>;

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "{0: <20} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
        "name", "wasm", "mem", "encoded", "gas"
    );

    report_gas::<_, _, ()>("minimal", Minimal, 32)?;
    report_gas::<NStack<u32, ()>, _, ()>(
        "minimal_nstack",
        Default::default(),
        32,
    )?;
    report_gas::<NStack<[u8; 64], ()>, _, ()>(
        "minimal_nstack_large",
        Default::default(),
        32,
    )?;
    report_gas::<Tree, (), ()>(
        "minimal_poseidon",
        Default::default(),
        Default::default(),
    )?;
    report_gas::<MockTransfer, (), ()>(
        "minimal_transfer",
        Default::default(),
        Default::default(),
    )?;
    Ok(())
}
