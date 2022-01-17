// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fibonacci::Fibonacci;
use microkelvin::{HostStore, StoreRef};
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};

fn get_config() -> Criterion {
    Criterion::default().sample_size(10)
}

fn fibonacci_15(
    network: &mut NetworkState,
    contract_id: ContractId,
    gas: &mut GasMeter,
) {
    let n: u64 = 15;

    network
        .query(contract_id, 0, fibonacci::ComputeFrom::new(n as u32), gas)
        .unwrap();
}

fn fibonacci_bench(c: &mut Criterion) {
    let code = include_bytes!(concat!(
        "../target/wasm32-unknown-unknown/release/",
        "fibonacci",
        ".wasm"
    ));

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&Fibonacci, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();
    let mut gas = GasMeter::with_limit(1_000_000_000_000);
    c.bench_function("fibonacci 15", |b| {
        b.iter(|| {
            fibonacci_15(
                black_box(&mut network),
                black_box(contract_id),
                black_box(&mut gas),
            )
        })
    });
}

criterion_main!(fibonacci_main);
criterion_group!(name = fibonacci_main; config = get_config(); targets = fibonacci_bench);
