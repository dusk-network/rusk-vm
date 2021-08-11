// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{Archive, Serialize};
use vm_poc::{Contract, Query, Queryable, Transactable, Transaction};

#[derive(Archive, Default, Serialize)]
pub struct Plutocracy {
    treasury: u64,
}

impl Contract for Plutocracy {
    fn code() -> &'static [u8] {
        include_bytes!(
            "../target/wasm32-unknown-unknown/release/plutocracy.wasm"
        )
    }
}

#[derive(Debug)]
pub struct TotalSupply;

impl Query for TotalSupply {
    const NAME: &'static str = "total_supply";
    type Return = u64;
}

impl Queryable<TotalSupply> for Plutocracy {
    fn query(&self, _arg: TotalSupply) -> u64 {
        self.treasury
    }
}

#[derive(Debug)]
pub struct Mint {
    pub amount: u64,
}

impl Transaction for Mint {
    const NAME: &'static str = "mint";
    type Return = ();
}

impl Transactable<Mint> for Plutocracy {
    fn transact(&mut self, mint: Mint) {
        self.treasury += mint.amount
    }
}

// to autogenerate

#[no_mangle]
fn total_supply(
    slf: &Plutocracy,
    q: TotalSupply,
) -> <TotalSupply as Query>::Return {
    slf.query(q)
}

#[no_mangle]
fn mint(slf: &mut Plutocracy, t: Mint) -> <Mint as Transaction>::Return {
    slf.transact(t)
}
