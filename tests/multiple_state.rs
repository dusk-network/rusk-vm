// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState, VMError};

struct TestCounter {
    network: NetworkState,
    contract_id: ContractId,
    gas: GasMeter,
}

impl TestCounter {
    pub fn new(value: i32) -> Self {
        let counter = Counter::new(value);

        let code = include_bytes!(
            "../target/wasm32-unknown-unknown/release/counter.wasm"
        );
        let contract = Contract::new(counter, code.to_vec());
        let mut network = NetworkState::new();
        let contract_id = network.deploy(contract).unwrap();

        let gas = GasMeter::with_limit(1_000_000_000);

        Self {
            gas,
            network,
            contract_id,
        }
    }

    pub fn network_mut(&mut self) -> &mut NetworkState {
        &mut self.network
    }

    pub fn read_value(&mut self) -> Result<i32, VMError> {
        self.network.query::<_, i32>(
            self.contract_id,
            0,
            counter::READ_VALUE,
            &mut self.gas,
        )
    }

    pub fn increment(&mut self) -> Result<(), VMError> {
        self.network.transact::<_, ()>(
            self.contract_id,
            0,
            counter::INCREMENT,
            &mut self.gas,
        )
    }
}

// This helper function setup an instance of `TestCounter` to have the following
// State in the Network:
// - Contract `counter` commited and pushed.
// - `origin`: with `counter` value at 99
// - `head`: with `counter` value at 100
// - `stage`: with `counter` value at 101
//
fn setup(counter: TestCounter) -> Result<TestCounter, VMError> {
    let mut counter = counter;
    assert_eq!(counter.read_value()?, 99, "Initial value is correct");

    let network = counter.network_mut();

    // This ensure the contract is part of all the three states of the network
    network.commit();
    network.push();

    // This shouldn't have any effect since we just commited and pushed.
    network.unstage();
    network.reset();

    assert_eq!(counter.read_value()?, 99, "Initial value is still correct");

    // Let's add some stage changes
    counter.increment()?;
    assert_eq!(counter.read_value()?, 100, "Value is incremented to 100");

    // Let's commit the changes
    counter.network_mut().commit();
    assert_eq!(counter.read_value()?, 100, "Value is still 100");

    // Let's add another stage changes
    counter.increment()?;
    assert_eq!(counter.read_value()?, 101, "Value is incremented to 101");

    Ok(counter)
}

#[test]
fn simple_unstage_reset() -> Result<(), VMError> {
    let mut counter = setup(TestCounter::new(99))?;

    // Recap:
    // - `origin`:value at 99
    // - `head`: value at 100
    // - `stage`: value at 101

    // Let's call `unstage`
    counter.network_mut().unstage();
    assert_eq!(counter.read_value()?, 100, "Value is now 100");

    // Let's call `reset`
    counter.network_mut().reset();
    assert_eq!(counter.read_value()?, 99, "Value is now 99");

    // Let's call `commit` and `push`: nothing should change now, since there
    // is no changes in the `staged` to be commited, and no changes in the
    // `head` to be pushed.
    let network = counter.network_mut();
    network.commit();
    network.push();
    assert_eq!(counter.read_value()?, 99, "Value is still 99");

    Ok(())
}

#[test]
fn push_with_staged() -> Result<(), VMError> {
    let mut counter = setup(TestCounter::new(99))?;

    // Recap:
    // - `origin`:value at 99
    // - `head`: value at 100
    // - `stage`: value at 101

    // Let's call `push`. This would lost `101` since it was never commited,
    // and makes `head` the new `origin`
    counter.network_mut().push();

    assert_eq!(counter.read_value()?, 100, "Value is now 100");

    // Let's call `unstage`: nothing should change now, there is no `stage`ed
    // changes.
    counter.network_mut().unstage();
    assert_eq!(counter.read_value()?, 100, "Value is still 100");

    // Let's call `reset`: nothing should change now, since `head` changes
    // were pushed to the `origin`
    counter.network_mut().reset();
    assert_eq!(counter.read_value()?, 100, "Value is still 100");

    // Let's call `commit` and `push`: nothing should change now, since there
    // is no changes in the `staged` to be commited, and no changes in the
    // `head` to be pushed.
    let network = counter.network_mut();
    network.commit();
    network.push();
    assert_eq!(counter.read_value()?, 100, "Value is still 100");

    Ok(())
}

#[cfg(feature = "persistence")]
#[test]
fn persist() -> Result<(), VMError> {
    use microkelvin::{BackendCtor, DiskBackend};
    fn testbackend() -> BackendCtor<DiskBackend> {
        BackendCtor::new(DiskBackend::ephemeral)
    }

    let mut counter = setup(TestCounter::new(99))?;

    // Recap:
    // - `origin`:value at 99
    // - `head`: value at 100
    // - `stage`: value at 101

    // Let's persist the state on disk...
    let id = counter
        .network_mut()
        .persist(&testbackend())
        .expect("State should be persisted");

    // and now restore it.
    *counter.network_mut() =
        NetworkState::new().restore(id).expect("State restored");

    // Since persistence does not include `staged` changes, the value should
    // be `100` instead of `101`.
    assert_eq!(counter.read_value()?, 100, "Value is now 100");

    // So, call `unstage` should have no effect.
    counter.network_mut().unstage();
    assert_eq!(counter.read_value()?, 100, "Value is still 100");

    // However, since persistence does include `head` changes, calling `reset`
    // should reset the value to `99`.
    counter.network_mut().reset();
    assert_eq!(counter.read_value()?, 99, "Value is now 99");

    Ok(())
}
