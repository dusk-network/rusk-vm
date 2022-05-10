// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use microkelvin::{OffsetLen, StoreRef};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Execute, Query};
use rusk_uplink_derive::{execute, init, query, state};

#[state]
pub struct Events;

#[init]
fn init() {}

#[query]
pub struct EventNum(pub u32);

impl Query for EventNum {
    const NAME: &'static str = "event_num";
    type Return = ();
}

#[execute(name = "event_num")]
impl Execute<EventNum> for Events {
    fn execute(&self, event_num: EventNum, store: StoreRef<OffsetLen>) {
        if event_num.0 > 0 {
            let callee = rusk_uplink::callee();

            rusk_uplink::query::<EventNum>(
                &callee,
                EventNum::new(event_num.0 - 1),
                0,
                store.clone(),
            )
            .unwrap();
        }
        rusk_uplink::emit("event_log", event_num.0, store);
    }
}
