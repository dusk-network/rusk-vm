// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use stack::*;

mod dual_test;
use dual_test::DualTest;

#[test]
fn stack() {
    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let mut test = DualTest::new(Stack::new(), code);

    type Leaf = u64;
    const N: Leaf = 16;

    for i in 0..N {
        test.apply(Push::new(i));
    }

    for i in 0..N {
        assert_eq!(test.execute(Peek::new(i)), Some(i))
    }

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(test.apply(Pop), Some(i))
    }

    assert_eq!(test.apply(Pop), None);
}
