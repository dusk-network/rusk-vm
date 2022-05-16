// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query, StoreContext};
use rusk_uplink_derive::{execute, query, state};

#[state]
pub struct Stringer;

#[query(new = false)]
pub struct Passthrough {
    string: String,
    repeat: u32,
    junk: u32,
}

impl Passthrough {
    pub fn new<S: Into<String>>(s: S, repeat: u32) -> Self {
        Passthrough {
            string: s.into(),
            junk: 82,
            repeat,
        }
    }
}

impl Query for Passthrough {
    const NAME: &'static str = "pass";
    type Return = String;
}

#[execute(name = "pass")]
impl Execute<Passthrough> for Stringer {
    fn execute(&self, p: Passthrough, _: StoreContext) -> String {
        p.string.repeat(p.repeat as usize)
    }
}
