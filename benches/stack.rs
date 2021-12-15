// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};
use stack::Stack;

fn get_config() -> Criterion {
    Criterion::default().sample_size(10)
}

fn stack_64(
    network: &mut NetworkState,
    contract_id: ContractId,
    gas: &mut GasMeter,
) {
    type Leaf = u64;
    const N: Leaf = 64;

    for i in 0..N {
        let _ =
            network.transact::<_, ()>(contract_id, 0, (stack::PUSH, i), gas);
    }
}

fn stack_bench(c: &mut Criterion) {
    type Leaf = u64;

    let stack = Stack::<Leaf>::new();

    let code = include_bytes!(concat!(
        "../target/wasm32-unknown-unknown/release/",
        "stack",
        ".wasm"
    ));

    let contract = Contract::new(stack, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();
    let mut gas = GasMeter::with_limit(1_000_000_000_000);
    c.bench_function("stack 64", |b| {
        b.iter(|| {
            stack_64(
                black_box(&mut network),
                black_box(contract_id),
                black_box(&mut gas),
            )
        })
    });
}

criterion_main!(stack_main);
criterion_group!(name = stack_main; config = get_config(); targets =
stack_bench);
