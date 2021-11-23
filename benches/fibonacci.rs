// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fibonacci::Fibonacci;
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};

fn get_config() -> Criterion {
    Criterion::default().sample_size(10)
}

fn fibonacci_3(
    network: &mut NetworkState,
    contract_id: ContractId,
    gas: &mut GasMeter,
) {
    let n: u64 = 3;

    network
        .query::<_, u64>(contract_id, 0, (fibonacci::COMPUTE, n), gas)
        .unwrap();
}

fn fibonacci_bench(c: &mut Criterion) {
    let code = include_bytes!(concat!(
        "../target/wasm32-unknown-unknown/release/",
        "fibonacci",
        ".wasm"
    ));

    let contract = Contract::new(Fibonacci, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();
    let mut gas = GasMeter::with_limit(1_000_000_000_000);
    c.bench_function("fibonacci 3", |b| {
        b.iter(|| {
            fibonacci_3(
                black_box(&mut network),
                black_box(contract_id),
                black_box(&mut gas),
            )
        })
    });
}

criterion_main!(fibonacci_main);
criterion_group!(name = fibonacci_main; config = get_config(); targets = fibonacci_bench);
