// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusk_vm::{Contract, ContractId, NetworkState};

fn get_config() -> Criterion {
    Criterion::default()
}

fn root_bench(c: &mut Criterion) {
    let mut network = NetworkState::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    for i in 0..u8::MAX {
        let counter = Counter::new(i as i32);
        let contract_id = ContractId::reserved(i);
        let contract = Contract::new(&counter, code.to_vec(), network.store());
        network
            .deploy_with_id(contract_id, contract)
            .expect("contract should be inserted successfully");
    }

    c.bench_function("root calculation", |b| {
        b.iter(|| black_box(network.root()))
    });
}

criterion_main!(root_main);
criterion_group!(name = root_main; config = get_config(); targets = root_bench);
