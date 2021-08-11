use vm_poc::*;

use plutocracy::{Mint, Plutocracy, TotalSupply};

#[test]
fn contract_standalone() {
    let mut pluto = Plutocracy::default();

    assert_eq!(pluto.query(TotalSupply), 0);

    pluto.transact(Mint { amount: 100 });

    assert_eq!(pluto.query(TotalSupply), 100);
}

#[test]
fn query_deployed_contract() {
    let mut state = State::default();

    let pluto = Plutocracy::default();

    let id = state.deploy(pluto);

    assert_eq!(state.query(id, TotalSupply).unwrap(), 0);
}

#[ignore]
#[test]
fn transact_deployed_contract() {
    let mut state = State::default();

    let pluto = Plutocracy::default();

    let id = state.deploy(pluto);

    assert_eq!(state.query(id, TotalSupply).unwrap(), 0);

    state.transact(id, Mint { amount: 100 }).unwrap();

    assert_eq!(state.query(id, TotalSupply).unwrap(), 100);
}
